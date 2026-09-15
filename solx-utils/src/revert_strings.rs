//!
//! Revert strings mode.
//!

use std::str::FromStr;

///
/// Revert strings mode.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RevertStrings {
    /// No compiler-generated strings; user-supplied ones are kept.
    Default,
    /// No compiler-generated strings; user-supplied ones are removed where possible.
    Strip,
    /// Strings for internal reverts; user-supplied ones are kept.
    Debug,
    /// Strings for internal reverts; user-supplied ones are added where absent.
    VerboseDebug,
}

impl FromStr for RevertStrings {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "default" => Self::Default,
            "strip" => Self::Strip,
            "debug" => Self::Debug,
            "verboseDebug" => Self::VerboseDebug,
            _ => anyhow::bail!(
                "Unsupported revert strings mode: {value}. Supported ones are: default, strip, debug, verboseDebug"
            ),
        })
    }
}
