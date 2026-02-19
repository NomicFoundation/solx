//!
//! MLIR context with dialect and translation registration.
//!

use melior::dialect::DialectRegistry;
use melior::ir::Module as MlirModule;
use melior::ir::operation::OperationLike;

use crate::llvm_module::LlvmModule;

/// MLIR context with all dialects and LLVM translation interfaces registered.
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
    /// Create a fully-initialized MLIR context with all dialects and LLVM
    /// translation interfaces registered.
    ///
    /// `register_all_llvm_translations` MUST be called before any MLIR-to-LLVM
    /// translation. Without it, `mlirTranslateModuleToLLVMIR` returns null.
    /// The constructor enforces this invariant.
    pub fn new() -> Self {
        // TODO: Register only the dialects we need (LLVM, arith, func, scf, cf)
        // instead of all upstream dialects.
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

        let context = melior::Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();
        melior::utility::register_all_llvm_translations(&context);

        Self { mlir: context }
    }

    /// Access the underlying `melior::Context`.
    pub fn mlir(&self) -> &melior::Context {
        &self.mlir
    }

    /// Translate MLIR source text (LLVM dialect) to a binary LLVM module.
    ///
    /// Parses the source, verifies it, and translates to LLVM IR.
    /// Returns an [`LlvmModule`] whose ownership can be transferred to
    /// inkwell via [`LlvmModule::into_raw`].
    pub fn try_into_llvm_module_from_source(&self, source: &str) -> Result<LlvmModule, String> {
        let module = MlirModule::parse(self.mlir(), source)
            .ok_or_else(|| "Failed to parse MLIR source text".to_string())?;

        self.try_into_llvm_module(&module)
    }

    /// Translate an MLIR module (LLVM dialect) to a binary `LLVMModuleRef`.
    ///
    /// No text serialization — returns the in-memory LLVM module directly.
    /// The context already has `register_all_llvm_translations` applied.
    pub fn try_into_llvm_module(&self, mlir_module: &MlirModule) -> Result<LlvmModule, String> {
        if !mlir_module.as_operation().verify() {
            return Err("MLIR module verification failed".into());
        }

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

#[cfg(test)]
mod tests {
    use melior::ir::operation::OperationLike;

    use super::*;

    #[test]
    fn context_creation() {
        let context = Context::new();
        let module =
            melior::ir::Module::parse(context.mlir(), "module {}").expect("MLIR should parse");
        assert!(module.as_operation().verify());
    }
}
