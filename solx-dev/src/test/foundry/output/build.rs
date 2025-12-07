//!
//! Foundry build output report.
//!

use std::collections::BTreeMap;

///
/// Foundry build output report. Is similar to `solc` standard JSON output.
///
#[derive(Debug, serde::Deserialize)]
pub struct Build {
    /// File-contract mapping.
    #[serde(default)]
    pub contracts: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    /// Compilation errors and warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<serde_json::Value>,
}
