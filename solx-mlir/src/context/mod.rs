//!
//! MLIR compilation context for EVM code generation.
//!

pub mod builder;
pub mod environment;
pub mod function;
pub mod pointer;
pub mod r#type;
pub mod value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Type;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::pass::PassManager;
use slang_solidity_v2::ast::NodeId;

use crate::llvm_module::RawLlvmModule;

use self::builder::Builder;
use self::function::Function;

/// Accumulated MLIR state threaded through the AST visitors.
pub struct Context<'context> {
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// Cached MLIR types and emission methods.
    pub builder: Builder<'context>,
    /// Resolution metadata keyed by the AST definition id of each function.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
    /// MLIR type of the contract being emitted (types `this`); set by the frontend before bodies.
    pub current_contract_type: Option<Type<'context>>,
    /// Virtual dispatch redirect: overridden base function id → most-derived override.
    pub virtual_redirect: HashMap<NodeId, NodeId>,
    /// Cross-contract references in encounter order, drained into the linker output.
    pub dependencies: RefCell<Vec<String>>,
}

impl<'context> Context<'context> {
    /// MLIR `builtin.module` operation name used to locate nested modules.
    const BUILTIN_MODULE: &'static str = "builtin.module";

    /// Creates a `melior::Context` with all upstream, Sol, and Yul dialects plus LLVM translations registered.
    pub fn create_mlir_context() -> melior::Context {
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

        let context = melior::Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();
        melior::utility::register_all_llvm_translations(&context);

        // Register Sol dialect passes once.
        static REGISTER_PASSES: Once = Once::new();
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        context
    }

    /// Creates a new MLIR state with an empty module carrying the EVM-version, data-layout, and target-triple attributes.
    pub fn new(context: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let location = Location::unknown(context);
        let module = Module::new(location);

        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                context.to_raw(),
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
            StringAttribute::new(context, target.data_layout()).into();
        let target_triple_attr: Attribute<'_> =
            StringAttribute::new(context, target.triple()).into();
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
            module,
            function_signatures: HashMap::new(),
            builder: Builder::new(context),
            current_contract_type: None,
            virtual_redirect: HashMap::new(),
            dependencies: RefCell::new(Vec::new()),
        }
    }

    /// Registers a function signature keyed by its AST definition id.
    pub fn register_function_signature(
        &mut self,
        definition_id: NodeId,
        mlir_name: String,
        parameter_types: Vec<Type<'context>>,
        return_types: Vec<Type<'context>>,
    ) {
        self.function_signatures.insert(
            definition_id,
            Function::new(mlir_name, parameter_types, return_types),
        );
    }

    /// Resolves a registered function by definition id (panics if unregistered — an internal invariant).
    pub fn resolve_function(&self, definition_id: NodeId) -> &Function<'context> {
        self.function_signatures
            .get(&definition_id)
            .unwrap_or_else(|| panic!("undefined function for definition {definition_id:?}"))
    }

    /// Redirects a virtual callee to its most-derived override (pass-through if not shadowed).
    pub fn resolve_virtual(&self, definition_id: NodeId) -> NodeId {
        self.virtual_redirect
            .get(&definition_id)
            .copied()
            .unwrap_or(definition_id)
    }

    /// Runs the Sol-to-LLVM conversion pass pipeline on `module` in place:
    /// canonicalize, modifier-op lowering, sol→yul, yul→std, canonicalize,
    /// scf→cf, func→llvm, arith→llvm, cf→llvm, reconcile-unrealized-casts.
    pub fn run_sol_passes(context: &melior::Context, module: &mut Module) -> anyhow::Result<()> {
        let pass_manager = PassManager::new(context);
        pass_manager.enable_verifier(true);

        // TODO: the canonicalizer causes an infinite loop on complex loop tests
        // (e.g. loop/complex/1.sol) at the -Oz optimization level.
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

    /// Runs the Sol-to-LLVM pipeline and splits the nested deploy/runtime modules
    /// into separate LLVM-dialect strings (the inner module matched by
    /// `runtime_code_identifier` is detached and stringified on its own).
    pub fn finalize_module(
        self,
        runtime_code_identifier: &str,
        capture_sol: bool,
    ) -> anyhow::Result<crate::output::MlirOutput> {
        let mut module = self.module;

        let sol_source = capture_sol.then(|| module.as_operation().to_string());

        Self::run_sol_passes(self.builder.context, &mut module)?;

        // Detach the runtime module so the deploy text doesn't duplicate it; the
        // deploy entry still references it via `evm.datasize`/`evm.dataoffset`.
        let runtime_llvm = Self::take_nested_module_text(&mut module, runtime_code_identifier)?;
        let deploy_llvm = module.as_operation().to_string();

        Ok(crate::output::MlirOutput {
            sol_source,
            deploy_source: deploy_llvm,
            runtime_source: runtime_llvm,
            dependencies: self.dependencies.into_inner(),
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
            if operation
                .name()
                .as_string_ref()
                .as_str()
                .expect("an MLIR operation name is valid UTF-8")
                != Self::BUILTIN_MODULE
            {
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

    /// Parses, verifies, and translates LLVM-dialect MLIR text to a raw LLVM module.
    pub fn translate_source_to_llvm(
        context: &melior::Context,
        source: &str,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(context, source)
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
                anyhow::bail!(
                    "mlirTranslateModuleToLLVMIR returned null — \
                     ensure register_all_llvm_translations was called"
                );
            }

            Ok(RawLlvmModule {
                context: llvm_context,
                module: llvm_module as *mut _,
            })
        }
    }
}
