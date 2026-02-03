//!
//! The compiler mode.
//!

pub mod imode;
pub mod llvm_options;

use std::collections::HashSet;
use std::fmt::Display;

use self::imode::IMode;
use self::imode::mode_to_string_aux;

use crate::compilers::llvm_ir::mode::Mode as LLVMMode;
use crate::compilers::solidity::solc::mode::Mode as SolcMode;
use crate::compilers::solidity::solx::mode::Mode as SolxMode;
use crate::compilers::yul::mode::Mode as YulMode;
use crate::compilers::yul::mode_upstream::Mode as YulUpstreamMode;

///
/// The compiler mode.
///
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Mode {
    /// The `solc` upstream mode.
    Solc(SolcMode),
    /// `solx` upstream mode.
    Solx(SolxMode),
    /// The `Yul` mode.
    Yul(YulMode),
    /// The `Yul` upstream mode.
    YulUpstream(YulUpstreamMode),
    /// The `LLVM` mode.
    LLVM(LLVMMode),
}

impl Mode {
    ///
    /// Checks if the mode is compatible with the filters.
    ///
    pub fn check_filters(&self, filters: &HashSet<String>) -> bool {
        filters.is_empty()
            || filters
                .iter()
                .any(|filter| self.normalize(filter).contains(filter))
    }

    ///
    /// Checks if the mode is compatible with the extended filters.
    /// The extended filter consists of 2 parts: mode substring and version range.
    ///
    pub fn check_extended_filters(&self, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        }

        for filter in filters.iter() {
            let mut split = filter.split_whitespace();

            let mode_filter = split.next().unwrap_or_default();
            let normalized_mode = self.normalize(mode_filter);
            if !normalized_mode.contains(mode_filter) {
                continue;
            }

            let version_or_optimizer_filter = match split.next() {
                Some(version) => version,
                None => return true,
            };
            if let Ok(version_req) = semver::VersionReq::parse(version_or_optimizer_filter) {
                if self.check_version(&version_req) {
                    return true;
                }
            } else {
                let normalized_mode = self.normalize(version_or_optimizer_filter);
                if !normalized_mode.contains(version_or_optimizer_filter) {
                    continue;
                }

                let version = match split.next() {
                    Some(version) => version,
                    None => return true,
                };
                if let Ok(version_req) = semver::VersionReq::parse(version)
                    && self.check_version(&version_req)
                {
                    return true;
                }
            }
        }

        false
    }

    ///
    /// Checks if the self is compatible with version filter.
    ///
    pub fn check_version(&self, versions: &semver::VersionReq) -> bool {
        let version = match self {
            Mode::Solc(mode) => &mode.solc_version,
            Mode::Solx(mode) => &mode.solc_version,
            _ => return false,
        };
        versions.matches(version)
    }

    ///
    /// Checks if the mode is compatible with the source code pragmas.
    ///
    pub fn check_pragmas(&self, sources: &[(String, String)]) -> bool {
        match self {
            Mode::Solc(mode) => mode.check_pragmas(sources),
            Mode::Solx(mode) => mode.check_pragmas(sources),
            _ => true,
        }
    }

    ///
    /// Checks if the mode is compatible with the Ethereum tests params.
    ///
    pub fn check_ethereum_tests_params(&self, params: &solx_solc_test_adapter::Params) -> bool {
        match self {
            Mode::Solc(mode) => mode.check_ethereum_tests_params(params),
            Mode::Solx(mode) => mode.check_ethereum_tests_params(params),
            _ => true,
        }
    }

    ///
    /// Returns the LLVM optimizer settings.
    ///
    pub fn llvm_optimizer_settings(&self) -> Option<&solx_codegen_evm::OptimizerSettings> {
        match self {
            Mode::Solc(_mode) => None,
            Mode::Solx(mode) => Some(&mode.llvm_optimizer_settings),
            Mode::Yul(mode) => Some(&mode.llvm_optimizer_settings),
            Mode::YulUpstream(_mode) => None,
            Mode::LLVM(mode) => Some(&mode.llvm_optimizer_settings),
        }
    }

    ///
    /// Returns the toolchain name for filtering.
    ///
    pub fn toolchain(&self) -> &'static str {
        match self {
            Mode::Solc(_mode) => "solc",
            Mode::Solx(_mode) => "solx",
            Mode::Yul(_mode) => "solx",
            Mode::YulUpstream(_mode) => "solc",
            Mode::LLVM(_mode) => "solx",
        }
    }

    ///
    /// Normalizes the mode according to the filter.
    ///
    fn normalize(&self, filter: &str) -> String {
        let mut current = self.to_string();
        if filter.contains("Y*") {
            current = regex::Regex::new("Y[-+]")
                .expect("Always valid")
                .replace_all(current.as_str(), "Y*")
                .to_string();
        }
        if filter.contains("E*") {
            current = regex::Regex::new("E[-+]")
                .expect("Always valid")
                .replace_all(current.as_str(), "E*")
                .to_string();
        }
        if filter.contains("I*") {
            current = regex::Regex::new("I[-+]")
                .expect("Always valid")
                .replace_all(current.as_str(), "I*")
                .to_string();
        }
        if filter.contains("V*") {
            current = regex::Regex::new("V[-+]")
                .expect("Always valid")
                .replace_all(current.as_str(), "V*")
                .to_string();
        }
        if filter.contains("M^") {
            current = regex::Regex::new("M[3z]")
                .expect("Always valid")
                .replace_all(current.as_str(), "M^")
                .to_string();
        }
        if filter.contains("M*") {
            current = regex::Regex::new("M[0123sz]")
                .expect("Always valid")
                .replace_all(current.as_str(), "M*")
                .to_string();
        }
        if filter.contains("B*") {
            current = regex::Regex::new("B[0123]")
                .expect("Always valid")
                .replace_all(current.as_str(), "B*")
                .to_string();
        }

        if filter.starts_with('^') {
            match self {
                Self::Solc(_) | Self::Solx(_) | Self::YulUpstream(_) => {
                    current = regex::Regex::new("[+]")
                        .expect("Always valid")
                        .replace_all(current.as_str(), "^")
                        .to_string();
                }
                Self::Yul(_) | Self::LLVM(_) => {
                    current = regex::Regex::new(".*M")
                        .expect("Always valid")
                        .replace_all(current.as_str(), "^M")
                        .to_string();
                }
            }
        }

        current = current.replace(' ', "");
        current
    }
}

impl From<SolcMode> for Mode {
    fn from(inner: SolcMode) -> Self {
        Self::Solc(inner)
    }
}

impl From<SolxMode> for Mode {
    fn from(inner: SolxMode) -> Self {
        Self::Solx(inner)
    }
}

impl From<YulMode> for Mode {
    fn from(inner: YulMode) -> Self {
        Self::Yul(inner)
    }
}

impl From<YulUpstreamMode> for Mode {
    fn from(inner: YulUpstreamMode) -> Self {
        Self::YulUpstream(inner)
    }
}

impl From<LLVMMode> for Mode {
    fn from(inner: LLVMMode) -> Self {
        Self::LLVM(inner)
    }
}

impl IMode for Mode {
    fn optimizations(&self) -> Option<String> {
        match self {
            Mode::Solc(mode) => mode.optimizations(),
            Mode::Solx(mode) => mode.optimizations(),
            Mode::Yul(mode) => mode.optimizations(),
            Mode::YulUpstream(mode) => mode.optimizations(),
            Mode::LLVM(mode) => mode.optimizations(),
        }
    }

    fn codegen(&self) -> Option<String> {
        match self {
            Mode::Solc(mode) => mode.codegen(),
            Mode::Solx(mode) => mode.codegen(),
            Mode::Yul(mode) => mode.codegen(),
            Mode::YulUpstream(mode) => mode.codegen(),
            Mode::LLVM(mode) => mode.codegen(),
        }
    }

    fn version(&self) -> Option<String> {
        match self {
            Mode::Solc(mode) => mode.version(),
            Mode::Solx(mode) => mode.version(),
            Mode::Yul(mode) => mode.version(),
            Mode::YulUpstream(mode) => mode.version(),
            Mode::LLVM(mode) => mode.version(),
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        mode_to_string_aux(self, f)
    }
}
