//!
//! `solx` Foundry project.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;

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
    /// Patch applied to the checkout, for test fixes the pinned commit lacks.
    #[serde(default)]
    pub patch: Option<PathBuf>,
    /// Build system that installs the project's packages, if it needs any.
    #[serde(default)]
    pub build_system: Option<BuildSystem>,
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
        build_systems: &BTreeMap<BuildSystem, String>,
        npm_version: &str,
        node_version: &str,
    ) -> anyhow::Result<String> {
        let patch = match self.patch.as_deref() {
            Some(path) => std::fs::read_to_string(path)
                .map_err(|error| anyhow::anyhow!("Patch {path:?} reading: {error}"))?,
            None => String::new(),
        };
        let (build_system, build_system_version, node_version) = match self.build_system {
            Some(build_system) => (
                build_system.to_string(),
                build_system.version(build_systems, npm_version),
                node_version,
            ),
            None => (String::new(), "", ""),
        };
        Ok([
            include_str!("../mod.rs"),
            include_str!("../../config/build_system.rs"),
            include_str!("../../../utils.rs"),
            self.url.as_str(),
            self.commit.as_deref().unwrap_or_default(),
            patch.as_str(),
            build_system.as_str(),
            build_system_version,
            node_version,
        ]
        .join("\0"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::BuildSystem;
    use super::Project;

    fn project(build_system: Option<BuildSystem>) -> Project {
        Project {
            description: String::new(),
            url: "https://github.com/owner/repository".to_owned(),
            commit: Some("0".repeat(40)),
            patch: None,
            build_system,
            rewrite_pragmas_in: Vec::new(),
            env: Default::default(),
            disabled: false,
        }
    }

    fn key(project: &Project, yarn_version: &str, npm_version: &str, node_version: &str) -> String {
        let build_systems = BTreeMap::from([
            (BuildSystem::Yarn, yarn_version.to_owned()),
            (BuildSystem::Bun, "1.0.0".to_owned()),
        ]);
        project
            .cache_key_material(&build_systems, npm_version, node_version)
            .expect("Always valid")
    }

    #[test]
    fn cache_key_material_covers_setup_inputs() {
        let patch_path = std::env::temp_dir().join(format!(
            "solx-dev-foundry-project-{}.patch",
            std::process::id()
        ));
        std::fs::write(patch_path.as_path(), "diff").expect("Always valid");
        let mut patched = project(Some(BuildSystem::Yarn));
        patched.patch = Some(patch_path.clone());

        let base = project(Some(BuildSystem::Yarn));
        let base_key = key(&base, "1.22.0", "11.0.0", "v24.0.0");
        let mut other_url = project(Some(BuildSystem::Yarn));
        other_url.url.push_str("-fork");
        let mut other_commit = project(Some(BuildSystem::Yarn));
        other_commit.commit = Some("1".repeat(40));
        let patched_key = key(&patched, "1.22.0", "11.0.0", "v24.0.0");
        std::fs::write(patch_path.as_path(), "other diff").expect("Always valid");
        let other_patch_key = key(&patched, "1.22.0", "11.0.0", "v24.0.0");
        std::fs::remove_file(patch_path.as_path()).expect("Always valid");
        assert_ne!(
            patched_key, other_patch_key,
            "the patch contents must change the key"
        );

        for (input, other) in [
            ("url", key(&other_url, "1.22.0", "11.0.0", "v24.0.0")),
            ("commit", key(&other_commit, "1.22.0", "11.0.0", "v24.0.0")),
            ("patch", patched_key),
            (
                "build system",
                key(
                    &project(Some(BuildSystem::Bun)),
                    "1.22.0",
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "build system version",
                key(&base, "1.22.1", "11.0.0", "v24.0.0"),
            ),
            ("node", key(&base, "1.22.0", "11.0.0", "v24.0.1")),
        ] {
            assert_ne!(base_key, other, "the {input} must change the key");
        }

        let npm_project = project(Some(BuildSystem::Npm));
        assert_ne!(
            key(&npm_project, "1.22.0", "11.0.0", "v24.0.0"),
            key(&npm_project, "1.22.0", "11.0.1", "v24.0.0"),
            "the npm version must change the key of an npm project"
        );
        assert_eq!(
            base_key,
            key(&base, "1.22.0", "11.0.1", "v24.0.0"),
            "the npm version must not change the key of a yarn project"
        );

        let without_packages = project(None);
        assert_eq!(
            key(&without_packages, "1.22.0", "11.0.0", "v24.0.0"),
            key(&without_packages, "1.22.1", "11.0.1", "v24.0.1"),
            "a project without packages must not depend on the build system and Node versions"
        );
    }
}
