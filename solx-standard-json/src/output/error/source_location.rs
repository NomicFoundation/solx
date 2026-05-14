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
    /// Start offset. [`SourceLocation::UNKNOWN_OFFSET`] when unknown.
    pub start: isize,
    /// End offset. [`SourceLocation::UNKNOWN_OFFSET`] when unknown.
    pub end: isize,
}

impl SourceLocation {
    /// Sentinel emitted in `start`/`end` when offsets are unknown,
    /// matching `solc`'s convention.
    pub const UNKNOWN_OFFSET: isize = -1;

    ///
    /// A shortcut constructor.
    ///
    pub fn new<S>(file: S, start: isize, end: isize) -> Self
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
