//!
//! `solx` Hardhat project.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::test::config::build_system::BuildSystem;

///
/// `solx` Hardhat project.
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
    /// Project build system.
    #[serde(default)]
    pub build_system: BuildSystem,
    /// Additional project dependencies.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Environment variables required for every command.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Build profile solx toolchains compile and test with.
    #[serde(default)]
    pub build_profile: Option<String>,
    /// Config installed as `hardhat.config.ts`, importing the project's own as `hardhat.config.base.<ext>`.
    #[serde(default)]
    pub config_overlay: Option<PathBuf>,
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
    ) -> String {
        let build_system_version = self.build_system.version(build_systems, npm_version);
        [
            include_str!("../../mod.rs"),
            include_str!("../../../config/build_system.rs"),
            include_str!("../../../../utils.rs"),
            self.url.as_str(),
            self.commit.as_deref().unwrap_or_default(),
            self.build_system.to_string().as_str(),
            build_system_version,
            self.dependencies.join(" ").as_str(),
            node_version,
        ]
        .join("\0")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::BuildSystem;
    use super::Project;

    fn project() -> Project {
        Project {
            description: String::new(),
            url: "https://github.com/owner/repository".to_owned(),
            commit: Some("0".repeat(40)),
            build_system: BuildSystem::Pnpm,
            dependencies: vec!["hardhat@3.0.0".to_owned()],
            env: Default::default(),
            build_profile: None,
            config_overlay: None,
            disabled: false,
        }
    }

    #[test]
    fn cache_key_material_covers_setup_inputs() {
        let build_systems = |pnpm_version: &str, yarn_version: &str| {
            BTreeMap::from([
                (BuildSystem::Yarn, yarn_version.to_owned()),
                (BuildSystem::Pnpm, pnpm_version.to_owned()),
                (BuildSystem::Bun, "1.0.0".to_owned()),
            ])
        };
        let key =
            project().cache_key_material(&build_systems("10.0.0", "1.22.0"), "11.0.0", "v24.0.0");
        let mut other_url = project();
        other_url.url.push_str("-fork");
        let mut other_commit = project();
        other_commit.commit = Some("1".repeat(40));
        let mut other_build_system = project();
        other_build_system.build_system = BuildSystem::Bun;
        let mut npm_project = project();
        npm_project.build_system = BuildSystem::Npm;
        let mut other_dependencies = project();
        other_dependencies.dependencies = vec!["hardhat@3.0.1".to_owned()];
        for (input, other) in [
            (
                "url",
                other_url.cache_key_material(
                    &build_systems("10.0.0", "1.22.0"),
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "commit",
                other_commit.cache_key_material(
                    &build_systems("10.0.0", "1.22.0"),
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "build system",
                other_build_system.cache_key_material(
                    &build_systems("10.0.0", "1.22.0"),
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "build system version",
                project().cache_key_material(
                    &build_systems("10.0.1", "1.22.0"),
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "dependencies",
                other_dependencies.cache_key_material(
                    &build_systems("10.0.0", "1.22.0"),
                    "11.0.0",
                    "v24.0.0",
                ),
            ),
            (
                "node",
                project().cache_key_material(
                    &build_systems("10.0.0", "1.22.0"),
                    "11.0.0",
                    "v24.0.1",
                ),
            ),
        ] {
            assert_ne!(key, other, "the {input} must change the key");
        }
        assert_ne!(
            npm_project.cache_key_material(&build_systems("10.0.0", "1.22.0"), "11.0.0", "v24.0.0"),
            npm_project.cache_key_material(&build_systems("10.0.0", "1.22.0"), "11.0.1", "v24.0.0"),
            "the npm version must change the key of an npm project"
        );

        assert_eq!(
            key,
            project().cache_key_material(&build_systems("10.0.0", "1.22.1"), "11.0.0", "v24.0.0"),
            "another build system's version must not change the key"
        );
        assert_eq!(
            key,
            project().cache_key_material(&build_systems("10.0.0", "1.22.0"), "11.0.1", "v24.0.0"),
            "the npm version must not change the key of a pnpm project"
        );
    }
}
