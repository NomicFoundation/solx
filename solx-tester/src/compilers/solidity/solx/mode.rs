//!
//! `solx` tester Solidity mode.
//!

use itertools::Itertools;

use crate::compilers::mode::llvm_options::LLVMOptions;

use crate::compilers::mode::imode::IMode;
use crate::compilers::mode::Mode as ModeWrapper;

///
/// `solx` tester Solidity mode.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mode {
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// Whether to enable the EVMLA codegen via Yul IR.
    pub via_ir: bool,
    /// Whether to enable the MLIR codegen.
    pub via_mlir: bool,
    /// The optimizer settings.
    pub llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings,
}

impl Mode {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: semver::Version,
        via_ir: bool,
        via_mlir: bool,
        mut llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings,
    ) -> Self {
        let llvm_options = LLVMOptions::get();
        llvm_optimizer_settings.is_verify_each_enabled = llvm_options.is_verify_each_enabled();
        llvm_optimizer_settings.is_debug_logging_enabled = llvm_options.is_debug_logging_enabled();

        Self {
            solc_version,
            via_ir,
            via_mlir,
            llvm_optimizer_settings,
        }
    }

    ///
    /// Unwrap mode.
    ///
    /// # Panics
    ///
    /// Will panic if the inner is non-Solidity mode.
    ///
    pub fn unwrap(mode: &ModeWrapper) -> &Self {
        match mode {
            ModeWrapper::Solx(mode) => mode,
            _ => panic!("Non-Solidity mode"),
        }
    }

    ///
    /// Checks if the mode is compatible with the source code pragmas.
    ///
    pub fn check_pragmas(&self, sources: &[(String, String)]) -> bool {
        sources.iter().all(|(_, source_code)| {
            match source_code.lines().find_map(|line| {
                let mut split = line.split_whitespace();
                if let (Some("pragma"), Some("solidity")) = (split.next(), split.next()) {
                    let version = split.join(",").replace(';', "");
                    semver::VersionReq::parse(version.as_str()).ok()
                } else {
                    None
                }
            }) {
                Some(pragma_version_req) => pragma_version_req.matches(&self.solc_version),
                None => true,
            }
        })
    }

    ///
    /// Checks if the mode is compatible with the Ethereum tests params.
    ///
    pub fn check_ethereum_tests_params(&self, params: &solx_solc_test_adapter::Params) -> bool {
        if self.via_ir {
            params.compile_via_yul != solx_solc_test_adapter::CompileViaYul::False
                && params.abi_encoder_v1_only != solx_solc_test_adapter::ABIEncoderV1Only::True
        } else {
            params.compile_via_yul != solx_solc_test_adapter::CompileViaYul::True
        }
    }
}

impl IMode for Mode {
    fn optimizations(&self) -> Option<String> {
        Some(format!("+{}", self.llvm_optimizer_settings))
    }

    fn codegen(&self) -> Option<String> {
        Some(
            if self.via_mlir {
                "L"
            } else if self.via_ir {
                "Y"
            } else {
                "E"
            }
            .to_string(),
        )
    }

    fn version(&self) -> Option<String> {
        Some(self.solc_version.to_string())
    }
}
