//!
//! `solc --standard-json` output error location.
//!

///
/// `solc --standard-json` output error location.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// File path.
    pub file: String,
    /// Start location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<isize>,
    /// End location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<isize>,
}

impl SourceLocation {
    ///
    /// A shortcut constructor.
    ///
    pub fn new<S>(file: S, start: Option<isize>, end: Option<isize>) -> Self
    where
        S: Into<String>,
    {
        Self {
            file: file.into(),
            start,
            end,
        }
    }
}
