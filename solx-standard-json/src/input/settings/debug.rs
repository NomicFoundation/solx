//!
//! The `solc --standard-json` input settings debug.
//!

///
/// The `solc --standard-json` input settings debug.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Debug {
    /// The revert strings setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert_strings: Option<String>,
}
