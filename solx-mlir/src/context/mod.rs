//!
//! MLIR context with dialect and translation registration.
//!

pub(crate) mod passes;
pub(crate) mod translation;

use std::sync::Once;

use melior::dialect::DialectRegistry;

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

    /// Access the underlying `melior::Context`.
    pub fn mlir(&self) -> &melior::Context {
        &self.mlir
    }
}
