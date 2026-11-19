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
