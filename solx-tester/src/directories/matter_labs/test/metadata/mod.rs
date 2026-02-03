//!
//! The Matter Labs compiler test metadata.
//!

pub mod case;

use std::collections::BTreeMap;
use std::str::FromStr;

use self::case::Case;

///
/// The Matter Labs compiler test metadata.
///
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Metadata {
    /// The test cases.
    pub cases: Vec<Case>,
    /// The mode filter.
    pub modes: Option<Vec<String>>,
    /// The toolchain filter.
    pub toolchains: Option<Vec<String>>,
    /// The test group.
    pub group: Option<String>,
    /// The test contracts as `instance -> path`.
    #[serde(default)]
    pub contracts: BTreeMap<String, String>,
    /// The test libraries for linking.
    #[serde(default)]
    pub libraries: BTreeMap<String, BTreeMap<String, String>>,
    /// If the entire test file must be ignored.
    #[serde(default)]
    pub ignore: bool,
}

impl FromStr for Metadata {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let json = string
            .lines()
            .filter_map(|line| {
                line.strip_prefix("//!")
                    .or_else(|| line.strip_prefix(";!"))
                    .or_else(|| line.strip_prefix("#!"))
            })
            .collect::<Vec<&str>>()
            .join("");

        serde_json::from_str(json.as_str()).or_else(|_| Ok(serde_json::from_str(string)?))
    }
}
