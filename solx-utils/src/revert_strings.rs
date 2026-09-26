//!
//! Revert strings mode.
//!

use std::str::FromStr;

///
/// Revert strings mode.
///
/// Mirrors `mlir::sol::RevertStrings` from the LLVM Sol dialect.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u32)]
pub enum RevertStrings {
    /// No compiler-generated strings; user-supplied ones are kept.
    Default = 0,
    /// No compiler-generated strings; user-supplied ones are removed where possible.
    Strip = 1,
    /// Strings for internal reverts; user-supplied ones are kept.
    Debug = 2,
    /// Strings for internal reverts; user-supplied ones are added where absent.
    VerboseDebug = 3,
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
