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
    ///
    pub fn cache_key_material(
        &self,
        yarn_version: &str,
        node_version: &str,
    ) -> anyhow::Result<String> {
        let patch = match self.patch.as_deref() {
            Some(path) => std::fs::read_to_string(path)
                .map_err(|error| anyhow::anyhow!("Patch {path:?} reading: {error}"))?,
            None => String::new(),
        };
        let (yarn_version, node_version) = if self.requires_yarn {
            (yarn_version, node_version)
        } else {
            ("", "")
        };
        Ok([
            include_str!("../mod.rs"),
            include_str!("../../../utils.rs"),
            self.url.as_str(),
            self.commit.as_deref().unwrap_or_default(),
            patch.as_str(),
            yarn_version,
            node_version,
        ]
        .join("\0"))
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
        let patch_path = std::env::temp_dir().join(format!(
            "solx-dev-foundry-project-{}.patch",
            std::process::id()
        ));
        std::fs::write(patch_path.as_path(), "diff").expect("Always valid");
        let mut patched = project(true);
        patched.patch = Some(patch_path.clone());

        let base = project(true);
        let key = base
            .cache_key_material("1.22.0", "v24.0.0")
            .expect("Always valid");
        let mut other_url = project(true);
        other_url.url.push_str("-fork");
        let mut other_commit = project(true);
        other_commit.commit = Some("1".repeat(40));
        let patched_key = patched
            .cache_key_material("1.22.0", "v24.0.0")
            .expect("Always valid");
        std::fs::write(patch_path.as_path(), "other diff").expect("Always valid");
        let other_patch_key = patched
            .cache_key_material("1.22.0", "v24.0.0")
            .expect("Always valid");
        std::fs::remove_file(patch_path.as_path()).expect("Always valid");
        assert_ne!(
            patched_key, other_patch_key,
            "the patch contents must change the key"
        );

        for (input, other) in [
            ("url", other_url.cache_key_material("1.22.0", "v24.0.0")),
            (
                "commit",
                other_commit.cache_key_material("1.22.0", "v24.0.0"),
            ),
            ("yarn", base.cache_key_material("1.22.1", "v24.0.0")),
            ("node", base.cache_key_material("1.22.0", "v24.0.1")),
        ] {
            assert_ne!(
                key,
                other.expect("Always valid"),
                "the {input} must change the key"
            );
        }
        assert_ne!(key, patched_key, "the patch must change the key");

        let without_yarn = project(false);
        assert_eq!(
            without_yarn
                .cache_key_material("1.22.0", "v24.0.0")
                .expect("Always valid"),
            without_yarn
                .cache_key_material("1.22.1", "v24.0.1")
                .expect("Always valid"),
            "a project without yarn must not depend on the yarn and Node versions"
        );
    }
}
