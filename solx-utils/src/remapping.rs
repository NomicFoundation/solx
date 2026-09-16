//!
//! The unified representation of a Solidity import remapping.
//!

use std::str::FromStr;

///
/// An import remapping in solc's `[context:]prefix=target` form.
///
#[derive(Debug, PartialEq)]
pub struct Remapping {
    /// Applies only within files whose identifier starts with this; empty matches every file.
    pub context: String,
    /// The import path prefix that `target` replaces.
    pub prefix: String,
    /// The replacement for `prefix`.
    pub target: String,
}

impl FromStr for Remapping {
    type Err = anyhow::Error;

    ///
    /// Follows solc's `ImportRemapper::parseRemapping`: the first `=` separates the target,
    /// the first `:` before it separates the context.
    ///
    fn from_str(remapping: &str) -> Result<Self, Self::Err> {
        let (rest, target) = remapping.split_once('=').ok_or_else(|| {
            anyhow::anyhow!("Remapping `{remapping}` target separator `=` is missing.")
        })?;
        let (context, prefix) = rest.split_once(':').unwrap_or(("", rest));
        if prefix.is_empty() {
            anyhow::bail!("Remapping `{remapping}` prefix is missing.");
        }
        Ok(Self {
            context: context.to_owned(),
            prefix: prefix.to_owned(),
            target: target.to_owned(),
        })
    }
}

impl std::fmt::Display for Remapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Always written: without it a prefix containing `:` would parse back as a context.
        write!(f, "{}:{}={}", self.context, self.prefix, self.target)
    }
}

impl serde::Serialize for Remapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Remapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let remapping = String::deserialize(deserializer)?;
        remapping.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Remapping;

    fn remapping(context: &str, prefix: &str, target: &str) -> Remapping {
        Remapping {
            context: context.to_owned(),
            prefix: prefix.to_owned(),
            target: target.to_owned(),
        }
    }

    #[test]
    fn parses_prefix_and_target() {
        assert_eq!(
            "@oz/=npm/oz@1.0.0/".parse::<Remapping>().unwrap(),
            remapping("", "@oz/", "npm/oz@1.0.0/")
        );
    }

    #[test]
    fn parses_context() {
        assert_eq!(
            "project/:@dep/=npm/dep@1.2.3/"
                .parse::<Remapping>()
                .unwrap(),
            remapping("project/", "@dep/", "npm/dep@1.2.3/")
        );
    }

    #[test]
    fn splits_context_at_the_first_colon() {
        assert_eq!(
            "a:b:c=d".parse::<Remapping>().unwrap(),
            remapping("a", "b:c", "d")
        );
    }

    #[test]
    fn splits_target_at_the_first_equals_sign() {
        assert_eq!(
            "a=b=c".parse::<Remapping>().unwrap(),
            remapping("", "a", "b=c")
        );
    }

    #[test]
    fn accepts_empty_target() {
        assert_eq!(
            "lib/=".parse::<Remapping>().unwrap(),
            remapping("", "lib/", "")
        );
    }

    #[test]
    fn rejects_missing_target_separator() {
        assert_eq!(
            "no-equals-sign"
                .parse::<Remapping>()
                .unwrap_err()
                .to_string(),
            "Remapping `no-equals-sign` target separator `=` is missing."
        );
    }

    #[test]
    fn rejects_missing_prefix() {
        for text in ["=target", ":=target"] {
            assert_eq!(
                text.parse::<Remapping>().unwrap_err().to_string(),
                format!("Remapping `{text}` prefix is missing.")
            );
        }
    }

    /// `solc --metadata` writes `settings.remappings` as `[":lib/=x/"]`.
    #[test]
    fn display_writes_solc_metadata_spelling() {
        assert_eq!(remapping("", "lib/", "x/").to_string(), ":lib/=x/");
        assert_eq!(
            remapping("project/", "@dep/", "npm/dep@1.2.3/").to_string(),
            "project/:@dep/=npm/dep@1.2.3/"
        );
    }
}
