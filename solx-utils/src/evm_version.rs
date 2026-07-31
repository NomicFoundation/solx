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

impl EVMVersion {
    ///
    /// Returns the LLVM subtarget feature string for the EVM version, or `None` for versions
    /// the EVM target does not model, which are its generic subtarget. Naming them anyway makes
    /// LLVM warn about an unrecognized feature for every function they are set on.
    ///
    /// The backend gates instructions with `Version >= <feature>` checks, so a version added
    /// after Osaka must keep advertising the features of every prior modelled version.
    ///
    pub fn llvm_target_features(self) -> Option<&'static str> {
        match self {
            Self::Cancun | Self::Prague => None,
            Self::Osaka => Some("+osaka"),
        }
    }
}

impl EVMVersion {
    /// Returns the Sol dialect `EvmVersionAttr` integer encoding.
    pub fn into_sol_dialect_identifier(self) -> u32 {
        match self {
            Self::Cancun => 11,
            Self::Prague => 12,
            Self::Osaka => 13,
        }
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
