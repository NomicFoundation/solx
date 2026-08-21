//!
//! MLIR compilation context for EVM code generation.
//!

pub mod contract;
pub mod environment;
pub mod function;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::CString;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Operation;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::ir::operation::OperationRef;
use melior::ir::operation::WalkOrder;
use melior::ir::operation::WalkResult;
use melior::pass::PassManager;

use crate::Block;
use crate::Type;
use crate::llvm_module::RawLlvmModule;

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated, and provides function registration, pass pipeline
/// execution, and LLVM translation. Emission is expressed through the [`crate::ir`] entity API.
pub struct Context<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub melior: &'context melior::Context,
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// The MLIR type of the contract currently being emitted, used to type
    /// `this` expressions. Frontends set this before emitting function bodies.
    pub current_contract_type: Option<Type<'context>>,
    /// The block value producers and effects append to during function-body emission. The
    /// control-flow emitters move it onto region entry blocks; it is absent between functions.
    pub current_block: Option<Block<'context>>,
}

impl<'context> Context<'context> {
    /// The op a code segment's module is.
    const BUILTIN_MODULE: &'static str = "builtin.module";
    /// The data layout the LLVM translation reads off the module.
    const DATA_LAYOUT: &'static str = "llvm.data_layout";
    /// The target triple the LLVM translation reads off the module.
    const TARGET_TRIPLE: &'static str = "llvm.target_triple";
    /// The EVM version the `convert-sol-to-yul` pass reads off the module.
    const EVM_VERSION: &'static str = "sol.evm_version";
    /// The attribute a `builtin.module` carries its own identifier in.
    const MODULE_SYMBOL: &'static str = "sym_name";

    /// The op the object-naming intrinsics reach as, after the Yul-to-standard pass.
    const LLVM_INTRINSIC_CALL: &'static str = "llvm.intrcall";
    /// The intrinsics naming an object: the segment's own code, the runtime child its deploy
    /// segment returns, and the contract a creation copies its bytecode from.
    const OBJECT_INTRINSICS: [&'static str; 2] = ["evm.dataoffset", "evm.datasize"];
    /// The attribute an intrinsic call carries its own name in.
    const INTRINSIC_NAME: &'static str = "name";
    /// The attribute an intrinsic call carries its string operands in.
    const INTRINSIC_METADATA: &'static str = "metadata";

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
        let mut module = Module::new(location);

        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                melior.to_raw(),
                evm_version.into_sol_dialect_identifier(),
            ))
        };
        let target = solx_utils::Target::EVM;
        let mut operation = module.as_operation_mut();
        operation.set_attribute(Self::EVM_VERSION, evm_version_attribute);
        operation.set_attribute(
            Self::DATA_LAYOUT,
            StringAttribute::new(melior, target.data_layout()).into(),
        );
        operation.set_attribute(
            Self::TARGET_TRIPLE,
            StringAttribute::new(melior, target.triple()).into(),
        );

        Self {
            melior,
            module,
            current_contract_type: None,
            current_block: None,
        }
    }

    /// The unknown source location.
    pub fn location(&self) -> Location<'context> {
        // TODO: stop generating unknown locations; every op should carry a real source
        // location derived from the AST going forward.
        Location::unknown(self.melior)
    }

    /// The block the insertion cursor points at, which the function-body emitters position before
    /// any emission.
    pub fn current_block(&self) -> Block<'context> {
        self.current_block
            .expect("the function body positions the insertion cursor")
    }

    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The pass pipeline is:
    /// 1. `canonicalize`
    /// 2. `sol-inline-modifiers`
    /// 3. `convert-sol-to-yul`: Sol → Yul
    /// 4. `convert-yul-to-std`: Yul → func/arith/scf/cf/LLVM, keeping the
    ///    memoryguard symbolic; the EVM backend's `evm-fold-memory-guard` pass
    ///    folds it on each spill retry
    /// 5. `canonicalize`
    /// 6. `convert-scf-to-cf`
    /// 7. `convert-func-to-llvm`
    /// 8. `convert-arith-to-llvm`
    /// 9. `convert-cf-to-llvm`
    /// 10. `reconcile-unrealized-casts`
    ///
    /// `sol-licm` is not yet in the pipeline.
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
                crate::ffi::mlirCreateSolModifierInliningPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToYulPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirSolCreateConvertYulToStandardPass(
                    /* symbolic_mem_guard = */ true,
                ),
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
        code_identifier: &str,
        capture_sol: bool,
    ) -> anyhow::Result<crate::output::MlirOutput> {
        let mut module = self.module;

        let sol_source = capture_sol.then(|| module.as_operation().to_string());

        Self::run_sol_passes(self.melior, &mut module)?;

        let runtime_code_identifier = format!(
            "{code_identifier}{}",
            solx_utils::Dependencies::DEPLOYED_OBJECT_SUFFIX
        );
        let (runtime_llvm, runtime_dependencies) =
            Self::take_nested_module(&mut module, runtime_code_identifier.as_str())?;
        let deploy_dependencies = Self::object_dependencies(
            &module.as_operation(),
            code_identifier,
            Some(runtime_code_identifier),
        );
        let deploy_llvm = module.as_operation().to_string();

        Ok(crate::output::MlirOutput {
            sol_source,
            deploy_source: deploy_llvm,
            deploy_dependencies,
            runtime_source: runtime_llvm,
            runtime_dependencies,
        })
    }

    /// Translate MLIR source text (LLVM dialect) to raw LLVM pointers.
    ///
    /// Parses the source, verifies it, lowers each `llvm.setimmutable` into heap stores at its
    /// id's `immutables` offsets, and translates to LLVM IR.
    /// Returns owned `(LLVMContextRef, LLVMModuleRef)`. The translated
    /// module still carries the symbolic memoryguard calls; the EVM backend
    /// folds them at the start of the optimization pipeline.
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, fails verification,
    /// or cannot be translated to LLVM IR.
    pub fn translate_source_to_llvm(
        melior: &melior::Context,
        source: &str,
        immutables: &BTreeMap<String, BTreeSet<u64>>,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(melior, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        let ids: Vec<CString> = immutables
            .keys()
            .map(|id| CString::new(id.as_str()).expect("an immutable id carries no NUL byte"))
            .collect();
        let (id_pointers, offsets): (Vec<*const std::ffi::c_char>, Vec<u64>) = ids
            .iter()
            .zip(immutables.values())
            .flat_map(|(id, offsets)| offsets.iter().map(|offset| (id.as_ptr(), *offset)))
            .unzip();

        unsafe {
            crate::ffi::mlirEvmLowerSetImmutables(
                module.to_raw(),
                id_pointers.as_ptr(),
                offsets.as_ptr(),
                offsets.len() as u64,
            );

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

    /// Finds a nested `builtin.module` in `module`'s body whose `sym_name` matches `target`,
    /// destroys it, and returns its textual form and the objects it references. The walk runs before
    /// the module leaves the tree, so each segment's dependencies come from its own code.
    fn take_nested_module(
        module: &mut Module,
        target: &str,
    ) -> anyhow::Result<(String, solx_utils::Dependencies)> {
        let body = module.body();
        std::iter::successors(body.first_operation_mut(), |operation| {
            operation.next_in_block_mut()
        })
        .find_map(|mut operation| {
            if operation.name().as_string_ref().as_str() != Ok(Self::BUILTIN_MODULE) {
                return None;
            }
            let symbol: StringAttribute = operation
                .attribute(Self::MODULE_SYMBOL)
                .ok()?
                .try_into()
                .ok()?;
            if symbol.value() != target {
                return None;
            }

            let dependencies = Self::object_dependencies(&operation, target, None);

            let text = operation.to_string();
            operation.remove_from_parent();
            drop(unsafe { Operation::from_raw(operation.to_raw()) });

            Some((text, dependencies))
        })
        .ok_or_else(|| anyhow::anyhow!("no module with sym_name `{target}` in Sol pass output"))
    }

    /// The objects `operation`'s code references, read off the intrinsics naming them.
    fn object_dependencies<'c: 'a, 'a>(
        operation: &impl OperationLike<'c, 'a>,
        identifier: &str,
        runtime: Option<String>,
    ) -> solx_utils::Dependencies {
        let mut dependencies = solx_utils::Dependencies::new(identifier, runtime);
        operation.walk(WalkOrder::PreOrder, |operation| {
            if let Some(object) = Self::referenced_object(operation) {
                dependencies.push(object);
            }
            WalkResult::Advance
        });
        dependencies
    }

    /// The object an `evm.dataoffset` / `evm.datasize` intrinsic call names.
    fn referenced_object(operation: OperationRef<'_, '_>) -> Option<String> {
        if operation.name().as_string_ref().as_str().ok()? != Self::LLVM_INTRINSIC_CALL {
            return None;
        }
        let name: StringAttribute = operation
            .attribute(Self::INTRINSIC_NAME)
            .ok()?
            .try_into()
            .ok()?;
        if !Self::OBJECT_INTRINSICS.contains(&name.value()) {
            return None;
        }
        let metadata = operation
            .attribute(Self::INTRINSIC_METADATA)
            .expect("an object intrinsic names its object in `metadata`");
        let object: StringAttribute =
            unsafe { Attribute::from_raw(mlir_sys::mlirArrayAttrGetElement(metadata.to_raw(), 0)) }
                .try_into()
                .expect("`metadata` is a one-element string array");
        Some(object.value().to_owned())
    }
}
