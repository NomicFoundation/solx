//!
//! Owned LLVM module produced by MLIR translation.
//!

/// Owned LLVM module produced by MLIR translation.
///
/// Wraps a raw `LLVMModuleRef` and its `LLVMContextRef`. Both are owned
/// and disposed on drop. Use `into_raw()` to transfer ownership to the
/// EVM backend pipeline.
pub struct LlvmModule {
    /// The owned LLVM module reference.
    module: inkwell::llvm_sys::prelude::LLVMModuleRef,
    /// The owned LLVM context reference.
    context: inkwell::llvm_sys::prelude::LLVMContextRef,
}

impl LlvmModule {
    /// Create from raw LLVM pointers.
    ///
    /// # Safety
    ///
    /// Caller must ensure both pointers are valid, owned, and not aliased.
    pub unsafe fn new(
        module: inkwell::llvm_sys::prelude::LLVMModuleRef,
        context: inkwell::llvm_sys::prelude::LLVMContextRef,
    ) -> Self {
        Self { module, context }
    }

    /// Consume self and return raw pointers without disposing.
    /// Use when handing ownership to inkwell or the EVM backend.
    pub fn into_raw(
        self,
    ) -> (
        inkwell::llvm_sys::prelude::LLVMContextRef,
        inkwell::llvm_sys::prelude::LLVMModuleRef,
    ) {
        let context = self.context;
        let module = self.module;
        std::mem::forget(self);
        (context, module)
    }
}

impl Drop for LlvmModule {
    fn drop(&mut self) {
        // SAFETY: `module` and `context` are owned raw pointers created in
        // `new()` and never aliased. Disposing them once on drop is sound.
        unsafe {
            inkwell::llvm_sys::core::LLVMDisposeModule(self.module);
            inkwell::llvm_sys::core::LLVMContextDispose(self.context);
        }
    }
}
