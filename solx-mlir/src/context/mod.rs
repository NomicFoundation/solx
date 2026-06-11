//!
//! MLIR compilation context for EVM code generation.
//!
//! Provides the [`Context`] type that owns the MLIR module and provides
//! helpers for creating common MLIR types, SSA naming, and function
//! registration. Emission methods live in the [`builder`] child module.
//!

pub mod builder;
pub mod environment;
pub mod function;
pub mod user_defined_operator;

pub use self::user_defined_operator::UserDefinedOperator;

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
///
/// Owns a `melior::ir::Module` being populated and provides helpers for
/// creating common MLIR types, SSA naming, and function registration.
/// Also provides pass pipeline execution and LLVM translation.
///
/// Mirrors the single-context pattern used by `solx-codegen-evm`.
pub struct Context<'context> {
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// Cached MLIR types and emission methods.
    pub builder: Builder<'context>,
    /// Resolution metadata keyed by the AST definition id of each function.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
    /// The MLIR type of the contract currently being emitted, used to type
    /// `this` expressions. Frontends set this before emitting function bodies.
    pub current_contract_type: Option<Type<'context>>,
    /// User-defined operator bindings (`using {f as op} for T global;`), keyed
    /// by `(udvt_definition_id, operator)` and mapping to the bound function's
    /// definition id. Frontends populate this before emitting bodies; a binary
    /// or unary operation on a bound user-defined value type then dispatches to
    /// the function rather than emitting native arithmetic.
    pub operator_bindings: HashMap<(NodeId, UserDefinedOperator), NodeId>,
    /// Redirect for `super` calls: maps a `super` member-access node id to the
    /// function node id it dispatches to under the most-derived contract's C3
    /// linearisation. Slang resolves `super` lexically, which is wrong in a
    /// diamond, so the frontend pre-resolves every `super` call against the
    /// linearised bases and records the result here. Empty unless the contract
    /// uses `super`.
    pub super_redirect: HashMap<NodeId, NodeId>,
    /// Virtual dispatch redirect: maps an overridden base function's node id to
    /// the most-derived override of the same signature. A plain internal call
    /// resolving (lexically) to a shadowed base function is routed through this
    /// so it reaches the override, matching Solidity's virtual semantics. Empty
    /// unless the contract overrides an inherited function.
    pub virtual_redirect: HashMap<NodeId, NodeId>,
    /// Cross-contract references collected during emission, in encounter order
    /// — populated when an emitter reaches into another contract (`new C(...)`
    /// → `sol.new`) and drained into the MLIR output for the linker. Empty
    /// unless the contract deploys another.
    pub dependencies: RefCell<Vec<String>>,
}

impl<'context> Context<'context> {
    // ---- Private constants ----

    /// MLIR `builtin.module` operation name used to locate nested modules.
    const BUILTIN_MODULE: &'static str = "builtin.module";

    // ==== Phase 1: Context creation ====

    /// Creates a fully-initialized `melior::Context` with all upstream
    /// dialects, Sol dialect, Yul dialect, and LLVM translation interfaces
    /// registered.
    ///
    /// `register_all_llvm_translations` MUST be called before any
    /// MLIR-to-LLVM translation. Without it, `mlirTranslateModuleToLLVMIR`
    /// returns null. This function enforces that invariant.
    pub fn create_mlir_context() -> melior::Context {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

        // SAFETY: FFI calls to register Sol and Yul dialects into the
        // registry. The registry and dialect handles are valid C objects
        // produced by the MLIR C API; no aliasing or lifetime issues.
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

        // Register Sol dialect passes so they can be added to a PassManager.
        // `Once` guarantees single-threaded, one-time execution.
        static REGISTER_PASSES: Once = Once::new();
        // SAFETY: `mlirRegisterSolPasses` is idempotent within a single
        // call but must not be called concurrently. `Once` provides full
        // happens-before ordering and guards against concurrent execution.
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        context
    }

    // ==== Phase 2: Module construction ====

    /// Creates a new MLIR state with an empty module.
    ///
    /// Sets the `sol.evm_version` module attribute required by the
    /// `convert-sol-to-yul` pass.
    pub fn new(context: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let location = Location::unknown(context);
        let module = Module::new(location);

        // Set the EVM version attribute on the module — required by the
        // Sol-to-Yul conversion pass.
        // SAFETY: `solxCreateEvmVersionAttr` returns a valid MlirAttribute
        // from the C++ Sol dialect. The context pointer is valid.
        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                context.to_raw(),
                evm_version.into_sol_dialect_identifier(),
            ))
        };
        // SAFETY: Setting a named attribute on the module operation. Both
        // the operation and attribute are valid MLIR objects owned by this
        // context.
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
        // SAFETY: Setting llvm.data_layout and llvm.target_triple on the
        // module. Both are string attributes required by the LLVM translation
        // layer. The module operation and attribute values are valid MLIR
        // objects owned by this context.
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
            operator_bindings: HashMap::new(),
            super_redirect: HashMap::new(),
            virtual_redirect: HashMap::new(),
            dependencies: RefCell::new(Vec::new()),
        }
    }

    /// Records a cross-contract reference (e.g. the object name passed to
    /// `sol.new`). Duplicates are ignored. The accumulated list is drained
    /// into [`crate::output::MlirOutput::dependencies`] at finalize time.
    pub fn add_dependency(&self, name: String) {
        let mut dependencies = self.dependencies.borrow_mut();
        if !dependencies.iter().any(|existing| existing == &name) {
            dependencies.push(name);
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
        let previous = self.function_signatures.insert(
            definition_id,
            Function::new(mlir_name, parameter_types, return_types),
        );
        debug_assert!(
            previous.is_none(),
            "duplicate function signature registration for definition {definition_id:?}",
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
        definition_id: NodeId,
    ) -> anyhow::Result<(&str, &[Type<'context>], &[Type<'context>])> {
        let function = self
            .function_signatures
            .get(&definition_id)
            .ok_or_else(|| {
                anyhow::anyhow!("undefined function for definition {definition_id:?}")
            })?;
        Ok((
            function.mlir_name.as_str(),
            &function.parameter_types,
            &function.return_types,
        ))
    }

    // ==== Phase 3: Sol pass pipeline ====

    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The pass pipeline is:
    /// 1. `convert-sol-to-yul` — Sol → Yul
    /// 2. `convert-yul-to-std` — Yul → func/arith/scf/cf/LLVM
    /// 3. `convert-scf-to-cf`
    /// 4. `convert-func-to-llvm`
    /// 5. `convert-arith-to-llvm`
    /// 6. `convert-cf-to-llvm`
    /// 7. `reconcile-unrealized-casts`
    ///
    /// Modifier lowering and LICM are skipped — they operate on `sol.modifier`
    /// and `sol.while`/`sol.for` ops which are not yet emitted.
    ///
    /// # Errors
    ///
    /// Returns an error if any pass in the pipeline fails.
    pub fn run_sol_passes(context: &melior::Context, module: &mut Module) -> anyhow::Result<()> {
        let pass_manager = PassManager::new(context);
        pass_manager.enable_verifier(true);

        // TODO: the canonicalizer pass causes an infinite loop on complex
        // loop tests (e.g. loop/complex/1.sol) at the -Oz optimization level.
        //
        // SAFETY: Each `mlirCreate*Pass` returns a freshly allocated pass
        // object. `Pass::from_raw` takes ownership. The pass manager runs
        // them sequentially on the module. No aliasing or use-after-free.
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
    /// The inner module (matched by `runtime_code_identifier`) is detached
    /// from the outer and stringified separately, so each can be translated
    /// to its own LLVM IR module by `solx-codegen-evm` and emit its own
    /// bytecode segment. The Sol-pass-generated outer carries the deploy
    /// entry that runs the constructor and returns the runtime bytecode —
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

        // Capture the Sol dialect MLIR before lowering, if requested.
        let sol_source = capture_sol.then(|| module.as_operation().to_string());

        // Lower Sol → LLVM dialect.
        Self::run_sol_passes(self.builder.context, &mut module)?;

        // Detach the inner runtime module so the deploy text doesn't carry
        // a duplicate copy and the deploy LLVM IR translation doesn't redo
        // runtime codegen. The deploy entry still references the runtime
        // via `evm.datasize`/`evm.dataoffset` metadata, which the linker
        // resolves through the runtime object's identifier.
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

    // ==== Phase 4: LLVM translation ====

    /// Translate MLIR source text (LLVM dialect) to raw LLVM pointers.
    ///
    /// Parses the source, verifies it, and translates to LLVM IR.
    /// Returns owned `(LLVMContextRef, LLVMModuleRef)`.
    ///
    /// `immutables` (the deploy segment's id -> reserved-offsets map, harvested
    /// from the runtime object) lowers every `llvm.setimmutable` to heap stores
    /// at those offsets before translation — `llvm.setimmutable` has no LLVM-IR
    /// translation, so a `ContractKind::Library`'s library-address immutable must
    /// be lowered here. `None` (the runtime segment, or a non-library) leaves the
    /// module unchanged (the runtime carries no `setimmutable`).
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, fails verification,
    /// or cannot be translated to LLVM IR.
    pub fn translate_source_to_llvm(
        context: &melior::Context,
        source: &str,
        immutables: Option<&std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>>,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(context, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        if let Some(immutables) = immutables {
            Self::lower_set_immutables(&module, immutables);
        }

        // SAFETY: `raw_operation` is a valid MlirOperation from a verified
        // module. `LLVMContextCreate` returns a fresh context. The LLVM
        // translation is safe because `register_all_llvm_translations` was
        // called in `create_mlir_context()`. Null-check guards the module
        // pointer.
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

    /// Lowers every `llvm.setimmutable` in `module` to heap stores at its
    /// immutable's reserved offsets (then erases it), via the solx-llvm C-API.
    /// The id -> offsets map is flattened to parallel arrays (one (id, offset)
    /// entry per pair), which the C-API rebuilds.
    fn lower_set_immutables(
        module: &Module,
        immutables: &std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>,
    ) {
        let mut id_cstrings: Vec<std::ffi::CString> = Vec::new();
        let mut offsets: Vec<u64> = Vec::new();
        for (id, id_offsets) in immutables {
            for &offset in id_offsets {
                id_cstrings.push(
                    std::ffi::CString::new(id.as_str()).expect("immutable id has no interior NUL"),
                );
                offsets.push(offset);
            }
        }
        let id_pointers: Vec<*const std::ffi::c_char> =
            id_cstrings.iter().map(|id| id.as_ptr()).collect();
        // SAFETY: `id_pointers` and `offsets` (parallel, length `offsets.len()`)
        // outlive the call; `id_cstrings` keeps the pointed-to bytes alive. The
        // module is a valid, verified MLIR module.
        unsafe {
            crate::ffi::mlirEvmLowerSetImmutables(
                module.to_raw(),
                id_pointers.as_ptr(),
                offsets.as_ptr(),
                offsets.len() as u64,
            );
        }
    }
}
