//!
//! MLIR compilation context for EVM code generation.
//!

pub mod contract;
pub mod environment;
pub mod function;

use std::collections::HashMap;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::pass::PassManager;
use slang_solidity_v2::ast::NodeId;

use crate::Type;
use crate::llvm_module::RawLlvmModule;

use self::function::Function;

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated, and provides function registration, pass pipeline
/// execution, and LLVM translation. Emission is expressed through the [`crate::ir`] entity API.
pub struct Context<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub melior: &'context melior::Context,
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// Resolution metadata keyed by the AST definition id of each function.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
    /// The MLIR type of the contract currently being emitted, used to type
    /// `this` expressions. Frontends set this before emitting function bodies.
    pub current_contract_type: Option<Type<'context>>,
}

impl<'context> Context<'context> {
    /// MLIR `builtin.module` operation name used to locate nested modules.
    const BUILTIN_MODULE: &'static str = "builtin.module";

    /// Creates a fully-initialized `melior::Context` with all upstream
    /// dialects, Sol dialect, Yul dialect, and LLVM translation interfaces
    /// registered.
    ///
    /// `register_all_llvm_translations` MUST be called before any
    /// MLIR-to-LLVM translation. Without it, `mlirTranslateModuleToLLVMIR`
    /// returns null. This function enforces that invariant.
    pub fn create_melior_context() -> melior::Context {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

        unsafe {
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__sol__(),
                registry.to_raw(),
            );
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__yul__(),
                registry.to_raw(),
            );
        }

        let melior = melior::Context::new();
        melior.append_dialect_registry(&registry);
        melior.load_all_available_dialects();
        melior::utility::register_all_llvm_translations(&melior);

        static REGISTER_PASSES: Once = Once::new();
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        melior
    }

    /// Creates a new MLIR state with an empty module.
    ///
    /// Sets the `sol.evm_version` module attribute required by the
    /// `convert-sol-to-yul` pass.
    pub fn new(melior: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let location = Location::unknown(melior);
        let module = Module::new(location);

        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                melior.to_raw(),
                evm_version.into_sol_dialect_identifier(),
            ))
        };
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"sol.evm_version".as_ptr()),
                evm_version_attribute.to_raw(),
            );
        }

        let target = solx_utils::Target::EVM;
        let data_layout_attr: Attribute<'_> =
            StringAttribute::new(melior, target.data_layout()).into();
        let target_triple_attr: Attribute<'_> =
            StringAttribute::new(melior, target.triple()).into();
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"llvm.data_layout".as_ptr()),
                data_layout_attr.to_raw(),
            );
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"llvm.target_triple".as_ptr()),
                target_triple_attr.to_raw(),
            );
        }

        Self {
            melior,
            module,
            function_signatures: HashMap::new(),
            current_contract_type: None,
        }
    }

    /// The unknown source location.
    pub fn location(&self) -> Location<'context> {
        Location::unknown(self.melior)
    }

    /// Registers a function signature keyed by its AST definition id.
    pub fn register_function_signature(
        &mut self,
        definition_node_id: NodeId,
        mlir_name: String,
        parameter_types: Vec<Type<'context>>,
        return_types: Vec<Type<'context>>,
    ) {
        let previous = self.function_signatures.insert(
            definition_node_id,
            Function::new(mlir_name, parameter_types, return_types),
        );
        debug_assert!(
            previous.is_none(),
            "duplicate function signature registration for definition {definition_node_id:?}",
        );
    }

    /// Resolves a function by its AST definition id.
    ///
    /// Returns the mangled MLIR name, declared parameter types, and return
    /// types.
    ///
    /// # Errors
    ///
    /// Returns an error if the definition was not registered.
    pub fn resolve_function(
        &self,
        definition_node_id: NodeId,
    ) -> anyhow::Result<(&str, &[Type<'context>], &[Type<'context>])> {
        let function = self
            .function_signatures
            .get(&definition_node_id)
            .ok_or_else(|| {
                anyhow::anyhow!("undefined function for definition {definition_node_id:?}")
            })?;
        Ok((
            function.mlir_name.as_str(),
            &function.parameter_types,
            &function.return_types,
        ))
    }

    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The pass pipeline is:
    /// 1. `convert-sol-to-yul`: Sol → Yul
    /// 2. `convert-yul-to-std`: Yul → func/arith/scf/cf/LLVM
    /// 3. `convert-scf-to-cf`
    /// 4. `convert-func-to-llvm`
    /// 5. `convert-arith-to-llvm`
    /// 6. `convert-cf-to-llvm`
    /// 7. `reconcile-unrealized-casts`
    ///
    /// Modifier lowering and LICM are skipped: they operate on `sol.modifier`
    /// and `sol.while`/`sol.for` ops which are not yet emitted.
    ///
    /// # Errors
    ///
    /// Returns an error if any pass in the pipeline fails.
    pub fn run_sol_passes(melior: &melior::Context, module: &mut Module) -> anyhow::Result<()> {
        let pass_manager = PassManager::new(melior);
        pass_manager.enable_verifier(true);

        unsafe {
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateSolModifierOpLoweringPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToYulPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertYulToStandardPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionSCFToControlFlowPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertFuncToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionArithToLLVMConversionPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertControlFlowToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionReconcileUnrealizedCastsPass(),
            ));
        }

        pass_manager
            .run(module)
            .map_err(|error| anyhow::anyhow!("Sol pass pipeline failed: {error}"))
    }

    /// Consumes the context, runs the Sol-to-LLVM pass pipeline, and returns
    /// the deploy and runtime modules as separate LLVM dialect strings.
    ///
    /// The Sol conversion pass produces a nested module:
    /// ```text
    /// module @Contract { deploy __entry + module @Contract_deployed { runtime __entry } }
    /// ```
    /// The inner module, matched by `runtime_code_identifier`, is detached
    /// from the outer and stringified separately, so each can be translated
    /// to its own LLVM IR module by `solx-codegen-evm` and emit its own
    /// bytecode segment. The Sol-pass-generated outer carries the deploy
    /// entry that runs the constructor and returns the runtime bytecode;
    /// it replaces the synthetic `minimal_deploy_code` wrapper that
    /// `solx-core` uses for non-MLIR pipelines.
    ///
    /// # Errors
    ///
    /// Returns an error if the pass pipeline fails or the runtime module
    /// is not found.
    pub fn finalize_module(
        self,
        runtime_code_identifier: &str,
        capture_sol: bool,
    ) -> anyhow::Result<crate::output::MlirOutput> {
        let mut module = self.module;

        let sol_source = capture_sol.then(|| module.as_operation().to_string());

        Self::run_sol_passes(self.melior, &mut module)?;

        let runtime_llvm = Self::take_nested_module_text(&mut module, runtime_code_identifier)?;
        let deploy_llvm = module.as_operation().to_string();

        Ok(crate::output::MlirOutput {
            sol_source,
            deploy_source: deploy_llvm,
            runtime_source: runtime_llvm,
        })
    }

    /// Finds a nested `builtin.module` in `module`'s body whose `sym_name`
    /// matches `target`, removes it from the parent, and returns its
    /// textual form.
    fn take_nested_module_text(module: &mut Module, target: &str) -> anyhow::Result<String> {
        let body = module.body();
        std::iter::successors(body.first_operation_mut(), |operation| {
            operation.next_in_block_mut()
        })
        .find_map(|mut operation| {
            if operation.name().as_string_ref().as_str().unwrap_or("") != Self::BUILTIN_MODULE {
                return None;
            }
            let symbol = operation.attribute("sym_name").ok()?;
            let symbol_name: StringAttribute = symbol.try_into().ok()?;
            if symbol_name.value() != target {
                return None;
            }
            let text = operation.to_string();
            operation.remove_from_parent();
            Some(text)
        })
        .ok_or_else(|| anyhow::anyhow!("no module with sym_name `{target}` in Sol pass output"))
    }

    /// Translate MLIR source text (LLVM dialect) to raw LLVM pointers.
    ///
    /// Parses the source, verifies it, and translates to LLVM IR.
    /// Returns owned `(LLVMContextRef, LLVMModuleRef)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, fails verification,
    /// or cannot be translated to LLVM IR.
    pub fn translate_source_to_llvm(
        melior: &melior::Context,
        source: &str,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(melior, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        unsafe {
            let raw_operation = module.as_operation().to_raw();
            let llvm_context = inkwell::llvm_sys::core::LLVMContextCreate();

            let llvm_module =
                mlir_sys::mlirTranslateModuleToLLVMIR(raw_operation, llvm_context as *mut _);

            if llvm_module.is_null() {
                inkwell::llvm_sys::core::LLVMContextDispose(llvm_context);
                anyhow::bail!("mlirTranslateModuleToLLVMIR returned null");
            }

            Ok(RawLlvmModule {
                context: llvm_context,
                module: llvm_module as *mut _,
            })
        }
    }
}
