//!
//! MLIR-to-LLVM IR translation.
//!

use melior::ir::Module as MlirModule;
use melior::ir::operation::OperationLike;

use crate::llvm_module::LlvmModule;

use crate::context::Context;

impl Context {
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
