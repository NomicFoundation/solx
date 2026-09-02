//!
//! The unified representation of a Solidity import remapping.
//!

///
/// An import remapping in solc's `[context:]prefix=target` form.
///
#[derive(Debug, PartialEq, Eq)]
pub struct Remapping {
    /// Applies only within files whose identifier starts with this; empty matches every file.
    pub context: String,
    /// The import path prefix that `target` replaces.
    pub prefix: String,
    /// The replacement for `prefix`.
    pub target: String,
}

impl TryFrom<&str> for Remapping {
    type Error = anyhow::Error;

    ///
    /// Parses solc's `[context:]prefix=target` form, as in solc's `ImportRemapper::parseRemapping`:
    /// the first `=` separates the target, the first `:` before it separates the context.
    ///
    fn try_from(remapping: &str) -> Result<Self, Self::Error> {
        let (rest, target) = remapping
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("Invalid remapping: \"{remapping}\""))?;
        let (context, prefix) = match rest.split_once(':') {
            Some((context, prefix)) => (context, prefix),
            None => ("", rest),
        };
        if prefix.is_empty() {
            anyhow::bail!("Invalid remapping: \"{remapping}\"");
        }
        Ok(Self {
            context: context.to_owned(),
            prefix: prefix.to_owned(),
            target: target.to_owned(),
        })
    }
}

impl std::fmt::Display for Remapping {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.context.is_empty() {
            write!(formatter, "{}:", self.context)?;
        }
        write!(formatter, "{}={}", self.prefix, self.target)
    }
}

#[cfg(test)]
mod tests {
    use super::Remapping;

    #[test]
    fn parse() {
        assert_eq!(
            Remapping::try_from("@oz/=npm/oz@1.0.0/").unwrap(),
            Remapping {
                context: "".to_owned(),
                prefix: "@oz/".to_owned(),
                target: "npm/oz@1.0.0/".to_owned(),
            }
        );
        assert_eq!(
            Remapping::try_from("project/:@dep/=npm/dep@1.2.3/").unwrap(),
            Remapping {
                context: "project/".to_owned(),
                prefix: "@dep/".to_owned(),
                target: "npm/dep@1.2.3/".to_owned(),
            }
        );
        // The prefix may contain colons past the first separator.
        assert_eq!(
            Remapping::try_from("a:b:c=d").unwrap(),
            Remapping {
                context: "a".to_owned(),
                prefix: "b:c".to_owned(),
                target: "d".to_owned(),
            }
        );
        // The target may contain equals signs past the first separator.
        assert_eq!(
            Remapping::try_from("a=b=c").unwrap(),
            Remapping {
                context: "".to_owned(),
                prefix: "a".to_owned(),
                target: "b=c".to_owned(),
            }
        );
        // An empty target erases the prefix.
        assert_eq!(
            Remapping::try_from("lib/=").unwrap(),
            Remapping {
                context: "".to_owned(),
                prefix: "lib/".to_owned(),
                target: "".to_owned(),
            }
        );
    }

    #[test]
    fn parse_invalid() {
        for remapping in ["no-equals-sign", "=target", ":=target"] {
            assert_eq!(
                Remapping::try_from(remapping).unwrap_err().to_string(),
                format!("Invalid remapping: \"{remapping}\"")
            );
        }
    }

    #[test]
    fn display_round_trips() {
        for remapping in [
            "@oz/=npm/oz@1.0.0/",
            "project/:@dep/=npm/dep@1.2.3/",
            "a:b:c=d=e",
            "lib/=",
        ] {
            assert_eq!(
                Remapping::try_from(remapping).unwrap().to_string(),
                remapping
            );
        }
    }
}
