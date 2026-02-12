//!
//! EVM version.
//!

use std::str::FromStr;

///
/// EVM version.
///
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum EVMVersion {
    /// The corresponding EVM version.
    #[serde(rename = "cancun")]
    Cancun,
    /// The corresponding EVM version.
    #[serde(rename = "prague")]
    Prague,
    /// The corresponding EVM version.
    #[serde(rename = "osaka")]
    #[default]
    Osaka,
}

impl FromStr for EVMVersion {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "cancun" => Self::Cancun,
            "prague" => Self::Prague,
            "osaka" => Self::Osaka,
            _ => anyhow::bail!(
                "Unsupported EVM version: {value}. Supported ones are: {}",
                vec![Self::Cancun, Self::Prague, Self::Osaka,]
                    .into_iter()
                    .map(|target| target.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        })
    }
}

impl std::fmt::Display for EVMVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancun => write!(f, "cancun"),
            Self::Prague => write!(f, "prague"),
            Self::Osaka => write!(f, "osaka"),
        }
    }
}
