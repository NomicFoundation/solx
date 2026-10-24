//!
//! `solx` Foundry project.
//!

use std::collections::HashMap;
use std::path::PathBuf;

///
/// `solx` Foundry project.
///
#[derive(Debug, serde::Deserialize)]
pub struct Project {
    /// Project description.
    #[serde(default)]
    pub description: String,
    /// Project URL.
    pub url: String,
    /// Git commit SHA to pin the repository to.
    #[serde(default)]
    pub commit: Option<String>,
    /// Patch applied to the checkout, for test fixes the pinned commit lacks.
    #[serde(default)]
    pub patch: Option<PathBuf>,
    /// Whether the project requires `yarn`.
    #[serde(default)]
    pub requires_yarn: bool,
    /// Installed package directories whose version pragmas are rewritten too.
    #[serde(default)]
    pub rewrite_pragmas_in: Vec<String>,
    /// Environment variables required for every command.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether the project is disabled.
    #[serde(default)]
    pub disabled: bool,
}

impl Project {
    ///
    /// Returns the inputs of the project setup, which key its cached checkout.
    ///
    /// The setup code is part of the key, so a harness change never reuses older setups.
    /// `patch` is the content of the file `self.patch` names.
    ///
    pub fn cache_key_material(
        &self,
        patch: &str,
        yarn_version: &str,
        node_version: &str,
    ) -> String {
        let (yarn_version, node_version) = if self.requires_yarn {
            (yarn_version, node_version)
        } else {
            ("", "")
        };
        [
            include_str!("../mod.rs"),
            include_str!("../../../utils.rs"),
            self.url.as_str(),
            self.commit.as_deref().unwrap_or_default(),
            patch,
            yarn_version,
            node_version,
        ]
        .join("\0")
    }
}

#[cfg(test)]
mod tests {
    use super::Project;

    fn project(requires_yarn: bool) -> Project {
        Project {
            description: String::new(),
            url: "https://github.com/owner/repository".to_owned(),
            commit: Some("0".repeat(40)),
            patch: None,
            requires_yarn,
            rewrite_pragmas_in: Vec::new(),
            env: Default::default(),
            disabled: false,
        }
    }

    #[test]
    fn cache_key_material_covers_setup_inputs() {
        let base = project(true);
        let key = base.cache_key_material("", "1.22.0", "v24.0.0");
        let mut other_url = project(true);
        other_url.url.push_str("-fork");
        let mut other_commit = project(true);
        other_commit.commit = Some("1".repeat(40));
        for (input, other) in [
            ("url", other_url.cache_key_material("", "1.22.0", "v24.0.0")),
            (
                "commit",
                other_commit.cache_key_material("", "1.22.0", "v24.0.0"),
            ),
            (
                "patch",
                base.cache_key_material("diff", "1.22.0", "v24.0.0"),
            ),
            ("yarn", base.cache_key_material("", "1.22.1", "v24.0.0")),
            ("node", base.cache_key_material("", "1.22.0", "v24.0.1")),
        ] {
            assert_ne!(key, other, "the {input} must change the key");
        }

        let without_yarn = project(false);
        assert_eq!(
            without_yarn.cache_key_material("", "1.22.0", "v24.0.0"),
            without_yarn.cache_key_material("", "1.22.1", "v24.0.1"),
            "a project without yarn must not depend on the yarn and Node versions"
        );
    }
}
