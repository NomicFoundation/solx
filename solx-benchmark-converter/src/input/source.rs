//!
//! Benchmark input source.
//!

use crate::output::comparison::Comparison;

///
/// Benchmark input source.
///
#[derive(Debug, Clone, Copy)]
pub enum Source {
    /// Tooling input source, e.g. Foundry or Hardhat.
    Tooling,
    /// `solx` tester input source.
    SolxTester,
}

impl Source {
    ///
    /// Returns the default comparisons for this source.
    ///
    /// These are legacy defaults that assume specific toolchain ordering.
    /// New code should use explicit comparisons from config files instead.
    ///
    pub fn default_comparisons(&self) -> Vec<Comparison> {
        match self {
            Self::Tooling => vec![
                Comparison::new(
                    "03.solx-viaIR".to_string(),
                    "01.solx-latest-viaIR".to_string(),
                ),
                Comparison::new(
                    "03.solx-legacy".to_string(),
                    "01.solx-latest-legacy".to_string(),
                ),
                Comparison::new(
                    "03.solx-viaIR".to_string(),
                    "02.solx-main-viaIR".to_string(),
                ),
                Comparison::new(
                    "03.solx-legacy".to_string(),
                    "02.solx-main-legacy".to_string(),
                ),
                Comparison::new(
                    "03.solx-viaIR".to_string(),
                    "00.solc-0.8.33-viaIR".to_string(),
                ),
                Comparison::new(
                    "03.solx-legacy".to_string(),
                    "00.solc-0.8.33-legacy".to_string(),
                ),
            ],
            Self::SolxTester => vec![
                Comparison::new(
                    "03.solx-viaIR".to_string(),
                    "02.solx-main-viaIR".to_string(),
                ),
                Comparison::new(
                    "03.solx-legacy".to_string(),
                    "02.solx-main-legacy".to_string(),
                ),
                Comparison::new(
                    "01.solx-latest-viaIR".to_string(),
                    "00.solc-0.8.33-viaIR".to_string(),
                ),
                Comparison::new(
                    "01.solx-latest-legacy".to_string(),
                    "00.solc-0.8.33-legacy".to_string(),
                ),
            ],
        }
    }
}

impl std::str::FromStr for Source {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "tooling" => Ok(Self::Tooling),
            "solx-tester" => Ok(Self::SolxTester),
            string => anyhow::bail!(
                "Unknown input source `{string}`. Supported values: {}",
                vec![Self::Tooling, Self::SolxTester]
                    .into_iter()
                    .map(|element| element.to_string().to_lowercase())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tooling => write!(f, "tooling"),
            Self::SolxTester => write!(f, "solx-tester"),
        }
    }
}
