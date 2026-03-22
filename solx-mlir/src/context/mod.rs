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

use std::collections::HashMap;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Type;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::IntegerType;
use melior::pass::PassManager;

use solx_utils::AddressSpace;

use self::builder::Builder;
use self::function::Function;

/// Raw LLVM module and context produced by MLIR translation.
///
/// The caller owns both pointers and must either pass them to inkwell
/// or dispose them manually.
pub struct RawLlvmModule {
    /// The LLVM context owning the module.
    pub context: inkwell::llvm_sys::prelude::LLVMContextRef,
    /// The LLVM module.
    pub module: inkwell::llvm_sys::prelude::LLVMModuleRef,
}

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated and provides helpers for
/// creating common MLIR types, SSA naming, and function registration.
/// Also provides pass pipeline execution and LLVM translation.
///
/// Mirrors the single-context pattern used by `solx-codegen-evm`.
pub struct Context<'context> {
    /// The MLIR module being built.
    module: Module<'context>,
    /// All function signatures for call resolution (bare name -> overloads).
    function_signatures: HashMap<String, Vec<Function>>,
    /// Cached MLIR types and emission methods.
    builder: Builder<'context>,
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
    /// `convert-sol-to-std` pass.
    pub fn new(context: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let location = Location::unknown(context);
        let module = Module::new(location);

        // Set the EVM version attribute on the module — required by the
        // Sol-to-standard conversion pass.
        // SAFETY: `solxCreateEvmVersionAttr` returns a valid MlirAttribute
        // from the C++ Sol dialect. The context pointer is valid.
        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                context.to_raw(),
                Self::sol_dialect_evm_version(evm_version),
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

        Self {
            module,
            function_signatures: HashMap::new(),
            builder: Builder {
                context,
                i256_type: IntegerType::new(context, solx_utils::BIT_LENGTH_FIELD as u32).into(),
                i1_type: IntegerType::new(context, 1).into(),
                sol_ptr_type: Type::parse(context, "!sol.ptr<i256, Stack>")
                    .expect("valid sol.ptr type syntax"),
                unknown_location: location,
            },
        }
    }

    /// Registers a function signature for call resolution.
    pub fn register_function_signature(
        &mut self,
        bare_name: &str,
        mlir_name: String,
        parameter_count: usize,
        return_count: usize,
    ) {
        self.function_signatures
            .entry(bare_name.to_owned())
            .or_default()
            .push(Function::new(mlir_name, parameter_count, return_count));
    }

    /// Resolves a function call by bare name and argument count.
    ///
    /// Returns the mangled MLIR name and the number of return values.
    ///
    /// # Errors
    ///
    /// Returns an error if the function is undefined or the call is ambiguous.
    pub fn resolve_function(
        &self,
        bare_name: &str,
        argument_count: usize,
    ) -> anyhow::Result<(&str, usize)> {
        let signatures = self
            .function_signatures
            .get(bare_name)
            .ok_or_else(|| anyhow::anyhow!("undefined function: {bare_name}"))?;
        // TODO: resolve overloads by parameter types, not just arity
        let matches: Vec<_> = signatures
            .iter()
            .filter(|signature| signature.parameter_count() == argument_count)
            .collect();
        match matches.len() {
            0 => anyhow::bail!("no overload of '{bare_name}' takes {argument_count} arguments"),
            1 => Ok((matches[0].mlir_name(), matches[0].return_count())),
            _ => {
                let overloads: Vec<&str> = matches
                    .iter()
                    .map(|signature| signature.mlir_name())
                    .collect();
                anyhow::bail!(
                    "ambiguous call to '{bare_name}' with {argument_count} arguments: {}",
                    overloads.join(", ")
                )
            }
        }
    }

    /// Returns a mutable reference to the underlying MLIR module.
    pub fn module_mut(&mut self) -> &mut Module<'context> {
        &mut self.module
    }

    /// Consumes the context and returns the accumulated MLIR module.
    pub fn into_module(self) -> Module<'context> {
        self.module
    }

    // ==== Phase 3: Sol pass pipeline ====

    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The pass pipeline is:
    /// 1. `convert-sol-to-std` — Sol + Yul → func/arith/scf/cf/LLVM
    /// 2. `convert-func-to-llvm`
    /// 3. `convert-scf-to-cf`
    /// 4. `convert-cf-to-llvm`
    /// 5. `convert-arith-to-llvm`
    /// 6. `reconcile-unrealized-casts`
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

        // SAFETY: Each `mlirCreate*Pass` returns a freshly allocated pass
        // object. `Pass::from_raw` takes ownership. The pass manager runs
        // them sequentially on the module. No aliasing or use-after-free.
        unsafe {
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToStandardPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertFuncToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionSCFToControlFlowPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertControlFlowToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionArithToLLVMConversionPass(),
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
    /// the resulting LLVM dialect MLIR as text.
    ///
    /// The Sol conversion pass produces a nested module structure:
    /// ```text
    /// module @Contract { deploy __entry + module @Contract_deployed { runtime __entry } }
    /// ```
    /// `solx-core` provides its own deploy wrapper, so this method extracts
    /// only the inner `_deployed` module and renames it to
    /// `runtime_code_identifier` so the LLVM module identifier matches what
    /// `minimal_deploy_code` references.
    ///
    /// # Errors
    ///
    /// Returns an error if re-parsing fails, the pass pipeline fails, or
    /// the deployed module is not found.
    pub fn finalize_module(self, runtime_code_identifier: &str) -> anyhow::Result<String> {
        // Re-parse the generated MLIR text to promote OperationBuilder
        // dictionary attributes to MLIR operation properties. Without this
        // round-trip, the Sol conversion pass fails because it expects
        // properties (not dict attrs) on operations like sol.func. This is
        // a melior limitation: OperationBuilder puts all attributes in the
        // dictionary, but the Sol C++ dialect defines them as properties.
        // The text format parser correctly separates them.
        let sol_text = self.module.as_operation().to_string();
        let mut parsed_module =
            Module::parse(self.builder.context, &sol_text).ok_or_else(|| {
                anyhow::anyhow!("failed to re-parse generated Sol dialect MLIR:\n{sol_text}")
            })?;

        // Lower Sol → LLVM dialect.
        Self::run_sol_passes(self.builder.context, &mut parsed_module)?;

        // Walk the outer module's body to find the inner `_deployed` module
        // and extract it as the runtime code. Rename it so the LLVM module
        // identifier matches what the deploy stub references.
        let body = parsed_module.body();
        let mut deployed_operation = None;
        let mut operation = body.first_operation();
        while let Some(current) = operation {
            if current.name().as_string_ref().as_str().unwrap_or("") == Self::BUILTIN_MODULE
                && let Ok(symbol) = current.attribute("sym_name")
            {
                let symbol_name: StringAttribute = symbol
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("sym_name is not a StringAttribute"))?;
                let symbol_name = symbol_name.value();
                if symbol_name == runtime_code_identifier {
                    deployed_operation = Some(current);
                    break;
                }
            }
            operation = current.next_in_block();
        }

        let runtime_op = deployed_operation
            .ok_or_else(|| anyhow::anyhow!("no _deployed module in Sol pass output"))?;

        Ok(runtime_op.to_string())
    }

    // ==== Phase 4: LLVM translation ====

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
        context: &melior::Context,
        source: &str,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(context, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
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

    // ==== Accessors ====

    /// Returns the builder for emitting MLIR operations.
    pub fn builder(&self) -> &Builder<'context> {
        &self.builder
    }

    /// Returns a reference to the melior context.
    pub fn context(&self) -> &'context melior::Context {
        self.builder.context
    }

    /// Returns the module body block for appending top-level operations.
    pub fn body(&self) -> BlockRef<'context, '_> {
        self.module.body()
    }

    /// Returns an unknown source location.
    pub fn location(&self) -> Location<'context> {
        self.builder.unknown_location
    }

    /// Returns the EVM word type (`i256`).
    pub fn i256(&self) -> Type<'context> {
        self.builder.i256_type
    }

    /// Returns the EVM boolean type (`i1`).
    pub fn i1(&self) -> Type<'context> {
        self.builder.i1_type
    }

    /// Returns an LLVM pointer type with the given address space.
    pub fn pointer(&self, address_space: AddressSpace) -> Type<'context> {
        melior::dialect::llvm::r#type::pointer(self.builder.context, address_space as u32)
    }

    // ---- Private ----

    /// Maps `solx_utils::EVMVersion` to the Sol dialect `EvmVersionAttr` `u32`
    /// encoding expected by the C++ backend.
    ///
    /// TODO: unify the MLIR constants with the existing enum
    fn sol_dialect_evm_version(version: solx_utils::EVMVersion) -> u32 {
        match version {
            solx_utils::EVMVersion::Cancun => 11,
            solx_utils::EVMVersion::Prague => 12,
            solx_utils::EVMVersion::Osaka => 13,
        }
    }
}
