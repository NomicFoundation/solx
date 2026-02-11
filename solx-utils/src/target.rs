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
    ///
    /// Returns the LLVM target triple.
    ///
    pub fn triple(&self) -> &str {
        match self {
            Self::EVM => "evm-unknown-unknown",
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
