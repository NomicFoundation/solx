//!
//! Hardhat test output report.
//!

pub mod stats;
pub mod test_result;

use std::path::PathBuf;

use self::stats::Stats;
use self::test_result::TestResult;

///
/// Hardhat test output report.
///
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Test {
    /// Total test statistics.
    #[serde(default)]
    pub stats: Stats,
    /// All test results.
    #[serde(default)]
    pub tests: Vec<TestResult>,
    /// Passed test results.
    #[serde(default)]
    pub passes: Vec<TestResult>,
    /// Failed test results.
    #[serde(default)]
    pub failures: Vec<TestResult>,
}

impl TryFrom<PathBuf> for Test {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let string = std::fs::read_to_string(path.as_path()).map_err(|error| {
            anyhow::anyhow!("Hardhat project report file {path:?} reading: {error}")
        })?;
        let report =
            solx_utils::deserialize_from_str::<Self>(string.as_str()).map_err(|error| {
                anyhow::anyhow!("Hardhat project report file {path:?} parsing: {error}")
            })?;
        Ok(report)
    }
}
