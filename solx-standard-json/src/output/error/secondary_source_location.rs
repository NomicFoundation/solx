//!
//! `solc --standard-json` output error secondary location.
//!

///
/// `solc --standard-json` output error secondary location.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecondarySourceLocation {
    /// File path.
    pub file: String,
    /// Start offset. [`super::source_location::SourceLocation::UNKNOWN_OFFSET`] when unknown.
    pub start: isize,
    /// End offset. [`super::source_location::SourceLocation::UNKNOWN_OFFSET`] when unknown.
    pub end: isize,
    /// Additional diagnostic message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
