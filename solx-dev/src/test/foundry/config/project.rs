//!
//! `solx` Foundry project.
//!

use std::collections::HashMap;

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
    /// Whether the project requires `yarn`.
    #[serde(default)]
    pub requires_yarn: bool,
    /// Environment variables required for every command.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether the project is disabled.
    #[serde(default)]
    pub disabled: bool,
}
