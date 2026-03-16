//!
//! MLIR context with dialect and translation registration.
//!

use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Module as MlirModule;
use melior::ir::operation::OperationLike;
use melior::pass::PassManager;

use crate::llvm_module::LlvmModule;

/// MLIR context with all dialects (including Sol and Yul) and LLVM
/// translation interfaces registered.
///
/// Owns a `melior::Context` that is ready for MLIR-to-LLVM translation.
/// Mirrors the `solx-codegen-evm::Context` pattern.
pub struct Context {
    /// The inner MLIR context with dialects and translations registered.
    mlir: melior::Context,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    /// Create a fully-initialized MLIR context with all upstream dialects,
    /// Sol dialect, Yul dialect, and LLVM translation interfaces registered.
    ///
    /// `register_all_llvm_translations` MUST be called before any MLIR-to-LLVM
    /// translation. Without it, `mlirTranslateModuleToLLVMIR` returns null.
    /// The constructor enforces this invariant.
    pub fn new() -> Self {
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
        // Only call once — double-registration may crash.
        static REGISTER_PASSES: Once = Once::new();
        // SAFETY: `mlirRegisterSolPasses` is idempotent within a single
        // call but must not be called concurrently. `Once` guarantees
        // single-threaded, one-time execution.
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        Self { mlir: context }
    }

    // ---- Public static ----

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
    pub fn run_sol_passes(
        context: &melior::Context,
        module: &mut MlirModule,
    ) -> Result<(), String> {
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
            .map_err(|e| format!("Sol pass pipeline failed: {e}"))
    }

    // ---- Public &self ----

    /// Access the underlying `melior::Context`.
    pub fn mlir(&self) -> &melior::Context {
        &self.mlir
    }

    /// Translate MLIR source text (LLVM dialect) to a binary LLVM module.
    ///
    /// Parses the source, verifies it, and translates to LLVM IR.
    /// Returns an [`LlvmModule`] whose ownership can be transferred to
    /// inkwell via [`LlvmModule::into_raw`].
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed, fails verification,
    /// or cannot be translated to LLVM IR.
    pub fn try_into_llvm_module_from_source(&self, source: &str) -> Result<LlvmModule, String> {
        let module = MlirModule::parse(self.mlir(), source)
            .ok_or_else(|| "Failed to parse MLIR source text".to_string())?;

        self.try_into_llvm_module(&module)
    }

    /// Run the Sol dialect conversion pipeline on a module, lowering all Sol
    /// and Yul ops to LLVM dialect, then translate to a binary LLVM module.
    ///
    /// See [`Self::run_sol_passes`] for the pass pipeline details.
    ///
    /// # Errors
    ///
    /// Returns an error if the pass pipeline fails or the resulting module
    /// cannot be translated to LLVM IR.
    pub fn try_into_llvm_module_from_sol(
        &self,
        module: &mut MlirModule,
    ) -> Result<LlvmModule, String> {
        Self::run_sol_passes(self.mlir(), module)?;
        self.try_into_llvm_module(module)
    }

    /// Translate an MLIR module (LLVM dialect) to a binary `LLVMModuleRef`.
    ///
    /// No text serialization — returns the in-memory LLVM module directly.
    /// The context already has `register_all_llvm_translations` applied.
    ///
    /// # Errors
    ///
    /// Returns an error if the module fails verification or the LLVM
    /// translation returns null.
    pub fn try_into_llvm_module(&self, mlir_module: &MlirModule) -> Result<LlvmModule, String> {
        if !mlir_module.as_operation().verify() {
            return Err("MLIR module verification failed".into());
        }

        // SAFETY: `raw_operation` is a valid MlirOperation from a verified
        // module. `LLVMContextCreate` returns a fresh context. The LLVM
        // translation is safe because `register_all_llvm_translations` was
        // called in `new()`. Null-check guards the module pointer.
        unsafe {
            let raw_operation = mlir_module.as_operation().to_raw();
            let llvm_context = inkwell::llvm_sys::core::LLVMContextCreate();

            // mlirTranslateModuleToLLVMIR: MlirOperation x LLVMContextRef -> LLVMModuleRef
            // mlir-sys and llvm-sys wrap the same C pointer types as distinct
            // Rust types — cast with `as *mut _` at the boundary.
            let llvm_module =
                mlir_sys::mlirTranslateModuleToLLVMIR(raw_operation, llvm_context as *mut _);

            if (llvm_module as *const std::ffi::c_void).is_null() {
                inkwell::llvm_sys::core::LLVMContextDispose(llvm_context);
                return Err("mlirTranslateModuleToLLVMIR returned null — \
                     ensure register_all_llvm_translations was called"
                    .into());
            }

            Ok(LlvmModule::new(llvm_module as *mut _, llvm_context))
        }
    }
}
