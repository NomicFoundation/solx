//!
//! Raw LLVM module produced by MLIR translation.
//!

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
