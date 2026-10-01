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
    /// Every supported version: not specified, or bounded from below by an unsupported one,
    /// which predates Cancun.
    #[default]
    Any,
    /// Equals specified.
    Equals(solx_utils::EVMVersion),
    /// Greater than specified.
    Greater(solx_utils::EVMVersion),
    /// Greater or equals than specified.
    GreaterEquals(solx_utils::EVMVersion),
    /// Lesser than specified.
    Lesser(solx_utils::EVMVersion),
    /// Lesser or equals than specified.
    LesserEquals(solx_utils::EVMVersion),
    /// No supported version: equal to or bounded from above by an unsupported one, which
    /// predates Cancun.
    Unsatisfiable,
}

impl EVMVersion {
    ///
    /// Checks whether the specified version matches the requirement.
    ///
    pub fn matches(&self, version: &solx_utils::EVMVersion) -> bool {
        match self {
            Self::Any => true,
            Self::Equals(inner) => version == inner,
            Self::Greater(inner) => version > inner,
            Self::GreaterEquals(inner) => version >= inner,
            Self::Lesser(inner) => version < inner,
            Self::LesserEquals(inner) => version <= inner,
            Self::Unsatisfiable => false,
        }
    }

    ///
    /// Returns the newest EVM version that matches the requirement, or `None` if no supported
    /// version does.
    ///
    pub fn newest_matching(&self) -> Option<solx_utils::EVMVersion> {
        [
            solx_utils::EVMVersion::Cancun,
            solx_utils::EVMVersion::Prague,
            solx_utils::EVMVersion::Osaka,
        ]
        .into_iter()
        .rev()
        .find(|version| self.matches(version))
    }
}

impl TryFrom<&str> for EVMVersion {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let regex = Regex::new(r"^(=|>|<|>=|<=)(\w+)$").expect("regex is compile-time constant");

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

        Ok(match (symbol, solx_utils::EVMVersion::from_str(version)) {
            (">" | ">=", Err(_)) => EVMVersion::Any,
            ("=", Ok(version)) => EVMVersion::Equals(version),
            (">", Ok(version)) => EVMVersion::Greater(version),
            (">=", Ok(version)) => EVMVersion::GreaterEquals(version),
            ("<", Ok(version)) => EVMVersion::Lesser(version),
            ("<=", Ok(version)) => EVMVersion::LesserEquals(version),
            ("=" | "<" | "<=", Err(_)) => EVMVersion::Unsatisfiable,
            (symbol, _) => anyhow::bail!("Invalid symbol before EVM version: {symbol}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::EVMVersion;

    fn newest_matching(requirement: &str) -> Option<solx_utils::EVMVersion> {
        EVMVersion::try_from(requirement)
            .expect("requirement is well-formed")
            .newest_matching()
    }

    #[test]
    fn malformed_requirement_is_rejected() {
        assert!(EVMVersion::try_from("~cancun").is_err());
    }

    #[test]
    fn lower_bound_below_cancun_resolves_to_osaka() {
        assert_eq!(
            newest_matching(">=byzantium"),
            Some(solx_utils::EVMVersion::Osaka)
        );
    }

    #[test]
    fn upper_bound_at_osaka_resolves_to_prague() {
        assert_eq!(
            newest_matching("<osaka"),
            Some(solx_utils::EVMVersion::Prague)
        );
    }

    #[test]
    fn equality_to_unsupported_version_matches_nothing() {
        assert_eq!(newest_matching("=shanghai"), None);
    }

    #[test]
    fn upper_bound_below_cancun_matches_nothing() {
        assert_eq!(newest_matching("<byzantium"), None);
    }
}
