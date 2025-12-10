//!
//! Hardhat test output report stats.
//!

///
/// Hardhat test output report stats.
///
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Stats {
    /// Total tests.
    #[serde(default)]
    pub tests: usize,
    /// Passed tests.
    #[serde(default)]
    pub passes: usize,
    /// Failed tests.
    #[serde(default)]
    pub failures: usize,
}
