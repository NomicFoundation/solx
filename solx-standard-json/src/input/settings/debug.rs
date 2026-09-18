//!
//! The `solc --standard-json` input settings debug.
//!

///
/// The `solc --standard-json` input settings debug.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Debug {
    /// The revert strings setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert_strings: Option<solx_utils::RevertStrings>,
}

impl From<solx_utils::RevertStrings> for Debug {
    fn from(revert_strings: solx_utils::RevertStrings) -> Self {
        Self {
            revert_strings: Some(revert_strings),
        }
    }
}
