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
    /// Caller must ensure both pointers are valid, owned, and not aliased.
    pub fn new(
        module: inkwell::llvm_sys::prelude::LLVMModuleRef,
        context: inkwell::llvm_sys::prelude::LLVMContextRef,
    ) -> Self {
        Self { module, context }
    }

    /// Raw `LLVMModuleRef` for passing to the EVM backend.
    ///
    /// The pointer is valid for the lifetime of this `LlvmModule`.
    pub fn as_raw(&self) -> inkwell::llvm_sys::prelude::LLVMModuleRef {
        self.module
    }

    /// Consume self and return raw pointers without disposing.
    /// Use when handing ownership to inkwell or the EVM backend.
    pub fn into_raw(
        self,
    ) -> (
        inkwell::llvm_sys::prelude::LLVMModuleRef,
        inkwell::llvm_sys::prelude::LLVMContextRef,
    ) {
        let module = self.module;
        let context = self.context;
        std::mem::forget(self);
        (module, context)
    }
}

impl Drop for LlvmModule {
    fn drop(&mut self) {
        unsafe {
            inkwell::llvm_sys::core::LLVMDisposeModule(self.module);
            inkwell::llvm_sys::core::LLVMContextDispose(self.context);
        }
    }
}
