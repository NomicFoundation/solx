//!
//! EVM version param values.
//!

use std::str::FromStr;

use regex::Regex;

///
/// EVM version param values.
///
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EVMVersion {
    /// Equals specified.
    Equals(solx_utils::EVMVersion),
    /// Greater than specified.
    Greater(solx_utils::EVMVersion),
    /// Lesser than specified.
    Lesser(solx_utils::EVMVersion),
    /// Greater or equals than specified.
    GreaterEquals(solx_utils::EVMVersion),
    /// Lesser or equals than specified.
    LesserEquals(solx_utils::EVMVersion),
    /// Not specified.
    #[default]
    Default,
}

impl EVMVersion {
    ///
    /// Checks whether the specified version matches the requirement.
    ///
    pub fn matches(&self, version: &solx_utils::EVMVersion) -> bool {
        match self {
            Self::Equals(inner) => version == inner,
            Self::Greater(inner) => version > inner,
            Self::Lesser(inner) => version < inner,
            Self::GreaterEquals(inner) => version >= inner,
            Self::LesserEquals(inner) => version <= inner,
            Self::Default => true,
        }
    }

    ///
    /// Returns the newest EVM version that matches the requirement.
    ///
    pub fn newest_matching(&self) -> solx_utils::EVMVersion {
        for version in [
            solx_utils::EVMVersion::Cancun,
            solx_utils::EVMVersion::Prague,
            solx_utils::EVMVersion::Osaka,
        ]
        .into_iter()
        .rev()
        {
            if self.matches(&version) {
                return version;
            }
        }
        solx_utils::EVMVersion::default()
    }
}

impl TryFrom<&str> for EVMVersion {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let regex = Regex::new(r"^(=|>|<|>=|<=)(\w*)$").expect("regex is compile-time constant");

        let captures = regex
            .captures(value)
            .ok_or_else(|| anyhow::anyhow!("Invalid EVM version description: {value}"))?;

        let symbol = captures
            .get(1)
            .expect("capture group 1 always present in matched pattern")
            .as_str();
        let version = captures
            .get(2)
            .expect("capture group 2 always present in matched pattern")
            .as_str();

        let version = solx_utils::EVMVersion::from_str(version)?;

        Ok(match symbol {
            "=" => EVMVersion::Equals(version),
            ">" => EVMVersion::Greater(version),
            "<" => EVMVersion::Lesser(version),
            ">=" => EVMVersion::GreaterEquals(version),
            "<=" => EVMVersion::LesserEquals(version),
            _ => anyhow::bail!("Invalid symbol before EVM version: {symbol}"),
        })
    }
}
