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
    /// Start location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<isize>,
    /// End location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<isize>,
    /// Additional diagnostic message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl SecondarySourceLocation {
    ///
    /// A shortcut constructor.
    ///
    pub fn new<S>(
        file: S,
        start: Option<isize>,
        end: Option<isize>,
        message: Option<String>,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            file: file.into(),
            start,
            end,
            message,
        }
    }
}
