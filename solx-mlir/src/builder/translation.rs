//!
//! MLIR-to-LLVM IR translation.
//!

use melior::ir::Module as MlirModule;
use melior::ir::operation::OperationLike;

use crate::builder::Context;
use crate::llvm_module::LlvmModule;

/// TODO: mirror solx-codegen-evm, move to src/context/mod.rs
impl<'context> Context<'context> {
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
    pub fn translate_source_to_llvm_module(
        context: &melior::Context,
        source: &str,
    ) -> anyhow::Result<LlvmModule> {
        let module = MlirModule::parse(context, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        Self::translate_module_to_llvm(&module)
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
    pub fn translate_sol_module_to_llvm(
        context: &melior::Context,
        module: &mut MlirModule,
    ) -> anyhow::Result<LlvmModule> {
        Self::run_sol_passes(context, module)?;
        Self::translate_module_to_llvm(module)
    }

    /// Translate an MLIR module (LLVM dialect) to a binary `LLVMModuleRef`.
    ///
    /// No text serialization — returns the in-memory LLVM module directly.
    /// The caller must ensure the `melior::Context` used to create the module
    /// has `register_all_llvm_translations` applied (guaranteed by
    /// [`Self::create_mlir_context`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the module fails verification or the LLVM
    /// translation returns null.
    pub fn translate_module_to_llvm(mlir_module: &MlirModule) -> anyhow::Result<LlvmModule> {
        if !mlir_module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        // SAFETY: `raw_operation` is a valid MlirOperation from a verified
        // module. `LLVMContextCreate` returns a fresh context. The LLVM
        // translation is safe because `register_all_llvm_translations` was
        // called in `create_mlir_context()`. Null-check guards the module
        // pointer.
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
                anyhow::bail!(
                    "mlirTranslateModuleToLLVMIR returned null — \
                     ensure register_all_llvm_translations was called"
                );
            }

            Ok(LlvmModule::new(llvm_module as *mut _, llvm_context))
        }
    }
}
