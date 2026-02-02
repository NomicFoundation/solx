//!
//! `solx` Hardhat config comparison.
//!

///
/// A comparison between two compilers for Excel diff columns.
///
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Comparison {
    /// The left compiler key (e.g., "solx").
    pub left: String,
    /// The right compiler key (e.g., "solc").
    pub right: String,
    /// Whether the comparison is disabled.
    #[serde(default)]
    pub disabled: bool,
}
