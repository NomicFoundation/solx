//!
//! `solx` Foundry project.
//!

use std::collections::HashMap;

use crate::test::config::build_system::BuildSystem;

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
    /// Build system that installs the project's packages, if it needs any.
    #[serde(default)]
    pub build_system: Option<BuildSystem>,
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
    pub fn cache_key_material(&self, build_system_version: &str, node_version: &str) -> String {
        let (build_system, build_system_version, node_version) = match self.build_system {
            Some(build_system) => (build_system.to_string(), build_system_version, node_version),
            None => (String::new(), "", ""),
        };
        [
            include_str!("../mod.rs"),
            include_str!("../../config/build_system.rs"),
            include_str!("../../../utils.rs"),
            self.url.as_str(),
            self.commit.as_deref().unwrap_or_default(),
            build_system.as_str(),
            build_system_version,
            node_version,
        ]
        .join("\0")
    }
}

#[cfg(test)]
mod tests {
    use super::BuildSystem;
    use super::Project;

    fn project(build_system: Option<BuildSystem>) -> Project {
        Project {
            description: String::new(),
            url: "https://github.com/owner/repository".to_owned(),
            commit: Some("0".repeat(40)),
            build_system,
            env: Default::default(),
            disabled: false,
        }
    }

    #[test]
    fn cache_key_material_covers_setup_inputs() {
        let base = project(Some(BuildSystem::Yarn));
        let key = base.cache_key_material("1.22.0", "v24.0.0");
        let mut other_url = project(Some(BuildSystem::Yarn));
        other_url.url.push_str("-fork");
        let mut other_commit = project(Some(BuildSystem::Yarn));
        other_commit.commit = Some("1".repeat(40));
        for (input, other) in [
            ("url", other_url.cache_key_material("1.22.0", "v24.0.0")),
            (
                "commit",
                other_commit.cache_key_material("1.22.0", "v24.0.0"),
            ),
            (
                "build system",
                project(Some(BuildSystem::Bun)).cache_key_material("1.22.0", "v24.0.0"),
            ),
            (
                "build system version",
                base.cache_key_material("1.22.1", "v24.0.0"),
            ),
            ("node", base.cache_key_material("1.22.0", "v24.0.1")),
        ] {
            assert_ne!(key, other, "the {input} must change the key");
        }

        let without_packages = project(None);
        assert_eq!(
            without_packages.cache_key_material("1.22.0", "v24.0.0"),
            without_packages.cache_key_material("1.22.1", "v24.0.1"),
            "a project without packages must not depend on the build system and Node versions"
        );
    }
}
