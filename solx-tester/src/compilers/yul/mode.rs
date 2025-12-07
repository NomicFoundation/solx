//!
//! `solx` tester Yul mode.
//!

use crate::compilers::mode::imode::IMode;
use crate::compilers::mode::llvm_options::LLVMOptions;

use crate::compilers::mode::Mode as ModeWrapper;

///
/// `solx` tester Yul mode.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mode {
    /// The optimizer settings.
    pub llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings,
}

impl Mode {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(mut llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings) -> Self {
        let llvm_options = LLVMOptions::get();
        llvm_optimizer_settings.enable_fallback_to_size();
        llvm_optimizer_settings.is_verify_each_enabled = llvm_options.is_verify_each_enabled();
        llvm_optimizer_settings.is_debug_logging_enabled = llvm_options.is_debug_logging_enabled();

        Self {
            llvm_optimizer_settings,
        }
    }

    ///
    /// Unwrap mode.
    ///
    /// # Panics
    ///
    /// Will panic if the inner is non-Yul mode.
    ///
    pub fn unwrap(mode: &ModeWrapper) -> &Self {
        match mode {
            ModeWrapper::Yul(mode) => mode,
            _ => panic!("Non-Yul mode"),
        }
    }
}

impl IMode for Mode {
    fn optimizations(&self) -> Option<String> {
        Some(format!("{}", self.llvm_optimizer_settings))
    }

    fn codegen(&self) -> Option<String> {
        None
    }

    fn version(&self) -> Option<String> {
        None
    }
}
