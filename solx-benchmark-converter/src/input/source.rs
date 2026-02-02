//!
//! Benchmark input source.
//!

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
