//!
//! EVM version.
//!

use std::str::FromStr;

///
/// EVM version.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum EVMVersion {
    /// The corresponding EVM version.
    #[serde(rename = "homestead")]
    Homestead,
    /// The corresponding EVM version.
    #[serde(rename = "tangerineWhistle")]
    TangerineWhistle,
    /// The corresponding EVM version.
    #[serde(rename = "spuriousDragon")]
    SpuriousDragon,
    /// The corresponding EVM version.
    #[serde(rename = "byzantium")]
    Byzantium,
    /// The corresponding EVM version.
    #[serde(rename = "constantinople")]
    Constantinople,
    /// The corresponding EVM version.
    #[serde(rename = "petersburg")]
    Petersburg,
    /// The corresponding EVM version.
    #[serde(rename = "istanbul")]
    Istanbul,
    /// The corresponding EVM version.
    #[serde(rename = "berlin")]
    Berlin,
    /// The corresponding EVM version.
    #[serde(rename = "london")]
    London,
    /// The corresponding EVM version.
    #[serde(rename = "paris")]
    Paris,
    /// The corresponding EVM version.
    #[serde(rename = "shanghai")]
    Shanghai,
    /// The corresponding EVM version.
    #[serde(rename = "cancun")]
    Cancun,
    /// The corresponding EVM version.
    #[serde(rename = "prague")]
    Prague,
    /// The corresponding EVM version.
    #[serde(rename = "osaka")]
    Osaka,
}

impl FromStr for EVMVersion {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "homestead" => Self::Homestead,
            "tangerineWhistle" => Self::TangerineWhistle,
            "spuriousDragon" => Self::SpuriousDragon,
            "byzantium" => Self::Byzantium,
            "constantinople" => Self::Constantinople,
            "petersburg" => Self::Petersburg,
            "istanbul" => Self::Istanbul,
            "berlin" => Self::Berlin,
            "london" => Self::London,
            "paris" => Self::Paris,
            "shanghai" => Self::Shanghai,
            "cancun" => Self::Cancun,
            "prague" => Self::Prague,
            "osaka" => Self::Osaka,
            _ => anyhow::bail!(
                "Unknown EVM version: {value}. Supported targets: {}",
                vec![
                    Self::Homestead,
                    Self::TangerineWhistle,
                    Self::SpuriousDragon,
                    Self::Byzantium,
                    Self::Constantinople,
                    Self::Petersburg,
                    Self::Istanbul,
                    Self::Berlin,
                    Self::London,
                    Self::Paris,
                    Self::Shanghai,
                    Self::Cancun,
                    Self::Prague,
                    Self::Osaka,
                ]
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
            Self::Homestead => write!(f, "homestead"),
            Self::TangerineWhistle => write!(f, "tangerineWhistle"),
            Self::SpuriousDragon => write!(f, "spuriousDragon"),
            Self::Byzantium => write!(f, "byzantium"),
            Self::Constantinople => write!(f, "constantinople"),
            Self::Petersburg => write!(f, "petersburg"),
            Self::Istanbul => write!(f, "istanbul"),
            Self::Berlin => write!(f, "berlin"),
            Self::London => write!(f, "london"),
            Self::Paris => write!(f, "paris"),
            Self::Shanghai => write!(f, "shanghai"),
            Self::Cancun => write!(f, "cancun"),
            Self::Prague => write!(f, "prague"),
            Self::Osaka => write!(f, "osaka"),
        }
    }
}
