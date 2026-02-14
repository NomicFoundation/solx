//!
//! `solx` Hardhat project.
//!

pub mod build_system;

use std::collections::HashMap;

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
    /// Project build system.
    #[serde(default)]
    pub build_system: BuildSystem,
    /// Additional project dependencies.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Environment variables required for every command.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether the project is disabled.
    #[serde(default)]
    pub disabled: bool,
}
