//!
//! Unified Yul mode for all toolchains.
//!

use crate::compilers::mode::Mode as ModeWrapper;
use crate::compilers::mode::imode::IMode;
use crate::compilers::mode::llvm_options::LLVMOptions;

///
/// Unified Yul mode for all toolchains.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mode {
    /// The Solidity compiler version (for solc toolchain).
    pub solc_version: Option<semver::Version>,
    /// The LLVM optimizer settings (for solx toolchain).
    pub llvm_optimizer_settings: Option<solx_codegen_evm::OptimizerSettings>,
    /// Whether to run the solc optimizer (for solc toolchain).
    pub solc_optimize: Option<bool>,
}

impl Mode {
    ///
    /// Creates a new mode for the solx toolchain.
    ///
    pub fn new_solx(mut llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings) -> Self {
        let llvm_options = LLVMOptions::get();
        llvm_optimizer_settings.enable_fallback_to_size();
        llvm_optimizer_settings.is_verify_each_enabled = llvm_options.is_verify_each_enabled();
        llvm_optimizer_settings.is_debug_logging_enabled = llvm_options.is_debug_logging_enabled();

        Self {
            solc_version: None,
            llvm_optimizer_settings: Some(llvm_optimizer_settings),
            solc_optimize: None,
        }
    }

    ///
    /// Creates a new mode for the solc toolchain.
    ///
    pub fn new_solc(solc_version: semver::Version, solc_optimize: bool) -> Self {
        Self {
            solc_version: Some(solc_version),
            llvm_optimizer_settings: None,
            solc_optimize: Some(solc_optimize),
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

    ///
    /// Returns whether this is a solx toolchain mode.
    ///
    pub fn is_solx(&self) -> bool {
        self.llvm_optimizer_settings.is_some()
    }
}

impl IMode for Mode {
    fn optimizations(&self) -> Option<String> {
        if let Some(ref llvm_settings) = self.llvm_optimizer_settings {
            Some(format!("{}", llvm_settings))
        } else {
            self.solc_optimize
                .map(|optimize| (if optimize { "+" } else { "-" }).to_string())
        }
    }

    fn codegen(&self) -> Option<String> {
        // Yul is always via Yul IR
        Some("Y".to_string())
    }

    fn version(&self) -> Option<String> {
        self.solc_version.as_ref().map(|v| v.to_string())
    }
}
