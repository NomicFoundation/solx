//!
//! `solx` Hardhat project.
//!

pub mod build_system;

use std::collections::HashMap;
use std::path::PathBuf;

use self::build_system::BuildSystem;

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
