//!
//! `solx` Foundry project.
//!

use std::collections::HashMap;

///
/// `solx` Foundry project.
///
#[derive(Debug, serde::Deserialize)]
pub struct Project {
    /// Project URL.
    pub url: String,
    /// Project description.
    pub description: String,
    /// Whether the project requires `yarn`.
    #[serde(default)]
    pub requires_yarn: bool,
    /// Environment variables required for every command.
    #[serde(default)]
    pub env: HashMap<String, String>,
}
