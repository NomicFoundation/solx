//!
//! Compilation target.
//!

///
/// Compilation target.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    /// The EVM target.
    EVM,
}

impl Target {
    /// Returns the LLVM target triple.
    pub fn triple(&self) -> &str {
        match self {
            Self::EVM => "evm-unknown-unknown",
        }
    }

    /// Returns the LLVM data layout string for the target.
    pub fn data_layout(&self) -> &str {
        match self {
            Self::EVM => "E-p:256:256-i256:256:256-S256-a:256:256",
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::EVM => write!(f, "evm"),
        }
    }
}
