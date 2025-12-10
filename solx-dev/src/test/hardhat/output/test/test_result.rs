//!
//! Hardhat test output report test result.
//!

///
/// Hardhat test output report test result.
///
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct TestResult {
    /// Title of the test.
    #[serde(default)]
    pub title: String,
    /// Full title of the test.
    #[serde(default)]
    pub full_title: String,
    /// File where the test is located.
    #[serde(default)]
    pub file: String,
    /// Errors encountered during the test, if any.
    #[serde(default)]
    pub err: serde_json::Value,
}
