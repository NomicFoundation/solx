//!
//! Unified Solidity mode for all toolchains.
//!

use itertools::Itertools;

use crate::compilers::mode::Mode as ModeWrapper;
use crate::compilers::mode::imode::IMode;
use crate::compilers::mode::llvm_options::LLVMOptions;

///
/// Unified Solidity mode for all toolchains.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mode {
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// Whether to enable the Yul IR path (true) or EVMLA path (false).
    pub via_ir: bool,
    /// The LLVM optimizer settings (used by solx toolchain).
    pub llvm_optimizer_settings: Option<solx_codegen_evm::OptimizerSettings>,
    /// Whether to run the solc optimizer (used by solc toolchain).
    pub solc_optimize: Option<bool>,
}

impl Mode {
    ///
    /// Creates a new mode for the solx toolchain.
    ///
    pub fn new_solx(
        solc_version: semver::Version,
        via_ir: bool,
        mut llvm_optimizer_settings: solx_codegen_evm::OptimizerSettings,
    ) -> Self {
        let llvm_options = LLVMOptions::get();
        llvm_optimizer_settings.is_verify_each_enabled = llvm_options.is_verify_each_enabled();
        llvm_optimizer_settings.is_debug_logging_enabled = llvm_options.is_debug_logging_enabled();

        Self {
            solc_version,
            via_ir,
            llvm_optimizer_settings: Some(llvm_optimizer_settings),
            solc_optimize: None,
        }
    }

    ///
    /// Creates a new mode for the solc toolchain.
    ///
    pub fn new_solc(solc_version: semver::Version, via_ir: bool, solc_optimize: bool) -> Self {
        Self {
            solc_version,
            via_ir,
            llvm_optimizer_settings: None,
            solc_optimize: Some(solc_optimize),
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
            ModeWrapper::Solidity(mode) => mode,
            _ => panic!("Non-Solidity mode"),
        }
    }

    ///
    /// Checks if the mode is compatible with the source code pragmas.
    ///
    pub fn check_pragmas(&self, sources: &[(String, String)]) -> bool {
        // The Slang frontend has no ABI coder v1: it parses `pragma abicoder v1`
        // as an inert node and always emits v2, so a source that explicitly
        // requests v1 can never reproduce its expected (v1) behaviour. Skip it,
        // mirroring solc's own source-pragma detection (libsolidity test util
        // `Common.cpp`). The `v2` guard keeps mixed-pragma v2 tests that merely
        // mention v1 in an inherited contract.
        #[cfg(feature = "slang-ast")]
        if sources
            .iter()
            .any(|(_, source_code)| source_code.contains("pragma abicoder v1"))
            && !sources.iter().any(|(_, source_code)| {
                source_code.contains("pragma abicoder v2")
                    || source_code.contains("pragma experimental ABIEncoderV2")
            })
        {
            return false;
        }

        // Strip pre-release for pragma matching since semver pre-release versions
        // have special matching rules that don't work well with Solidity pragmas.
        // E.g., ">=0.8.0" should match "0.8.34-develop" but semver doesn't agree.
        let version_for_pragma = semver::Version::new(
            self.solc_version.major,
            self.solc_version.minor,
            self.solc_version.patch,
        );

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
                Some(pragma_version_req) => pragma_version_req.matches(&version_for_pragma),
                None => true,
            }
        })
    }

    ///
    /// Checks if the mode is compatible with the Ethereum tests params.
    ///
    pub fn check_ethereum_tests_params(&self, params: &solx_solc_test_adapter::Params) -> bool {
        // The Slang frontend has no ABI coder v1. Slang parses `pragma abicoder
        // v1` but models no v1 semantics — the pragma is an inert AST node, and
        // Slang's bindings, type-checker, and ABI computation are all
        // version-agnostic — so solx-slang always emits v2. `ABIEncoderV1Only`
        // tests exercise v1-specific behavior that can never reproduce under the
        // Slang pipeline, so skip them entirely (the `via_ir` branch below
        // already excludes them for the Yul pipeline).
        #[cfg(feature = "slang-ast")]
        if params.abi_encoder_v1_only == solx_solc_test_adapter::ABIEncoderV1Only::True {
            return false;
        }
        if self.via_ir {
            params.compile_via_yul != solx_solc_test_adapter::CompileViaYul::False
                && params.abi_encoder_v1_only != solx_solc_test_adapter::ABIEncoderV1Only::True
        } else {
            params.compile_via_yul != solx_solc_test_adapter::CompileViaYul::True
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
        self.llvm_optimizer_settings
            .as_ref()
            .map(|settings| format!("{settings}"))
    }

    fn codegen(&self) -> Option<String> {
        Some((if self.via_ir { "Y" } else { "E" }).to_string())
    }

    fn version(&self) -> Option<String> {
        Some(self.solc_version.to_string())
    }
}
