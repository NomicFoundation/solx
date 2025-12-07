//!
//! `solx` tester upstream Yul mode.
//!

use crate::compilers::mode::{imode::IMode, Mode as ModeWrapper};

///
/// `solx` tester upstream Yul mode.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Mode {
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// Whether to enable the MLIR codegen.
    pub via_mlir: bool,
    /// Whether to run the Solidity compiler optimizer.
    pub solc_optimize: bool,
}

impl Mode {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(solc_version: semver::Version, via_mlir: bool, solc_optimize: bool) -> Self {
        Self {
            solc_version,
            via_mlir,
            solc_optimize,
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
            ModeWrapper::YulUpstream(mode) => mode,
            _ => panic!("Non-Yul-upstream mode"),
        }
    }
}

impl IMode for Mode {
    fn optimizations(&self) -> Option<String> {
        Some((if self.solc_optimize { "+" } else { "-" }).to_string())
    }

    fn codegen(&self) -> Option<String> {
        Some((if self.via_mlir { "L" } else { "Y" }).to_string())
    }

    fn version(&self) -> Option<String> {
        Some(format!("{}", self.solc_version))
    }
}
