//!
//! `solc --standard-json` output error.
//!

pub mod collectable;
pub mod secondary_source_location;
pub mod source_location;

use std::collections::BTreeMap;

use crate::input::source::Source as InputSource;

use self::secondary_source_location::SecondarySourceLocation;
use self::source_location::SourceLocation;

///
/// `solc --standard-json` output error.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    /// The component type.
    pub component: String,
    /// The error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// The code of the Slang diagnostic, kept apart from `error_code`, which tools parse as a number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slang_error_code: Option<String>,
    /// The formatted error message.
    pub formatted_message: String,
    /// The non-formatted error message.
    pub message: String,
    /// The error severity.
    pub severity: String,
    /// The error location data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    /// The error secondary location data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_source_location: Option<SecondarySourceLocation>,
    /// The error type.
    pub r#type: String,
}

impl Error {
    ///
    /// A shortcut constructor.
    ///
    pub fn new<S>(
        path: Option<&str>,
        r#type: &str,
        error_code: Option<&str>,
        message: S,
        source_location: Option<SourceLocation>,
        sources: Option<&BTreeMap<String, InputSource>>,
    ) -> Self
    where
        S: std::fmt::Display,
    {
        let message = message.to_string();

        let message_trimmed = message.trim();
        let mut formatted_message = if message_trimmed.starts_with(r#type) {
            message_trimmed.to_owned()
        } else {
            format!("{type}: {message_trimmed}")
        };
        formatted_message.push('\n');
        if let Some(source_location) = source_location.as_ref() {
            let path = path.unwrap_or(source_location.file.as_str());
            let source_code =
                sources.and_then(|sources| sources.get(path).and_then(|source| source.content()));
            let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
                path.to_owned(),
                source_location.start,
                source_location.end,
                source_code,
            );
            formatted_message.push_str(mapped_location.to_string().as_str());
            formatted_message.push('\n');
        }

        Self {
            component: "general".to_owned(),
            error_code: error_code.map(str::to_owned),
            slang_error_code: None,
            formatted_message,
            message,
            severity: r#type.to_lowercase(),
            source_location,
            secondary_source_location: None,
            r#type: r#type.to_owned(),
        }
    }

    ///
    /// Attaches the code of the Slang diagnostic the error reports.
    ///
    pub fn with_slang_error_code(mut self, slang_error_code: &str) -> Self {
        self.slang_error_code = Some(slang_error_code.to_owned());
        self
    }

    ///
    /// Creates a new simple error
    ///
    pub fn new_error<S>(message: S) -> Self
    where
        S: std::fmt::Display,
    {
        Self::new_error_with_data(None, None, message, None, None)
    }

    ///
    /// Creates a new simple warning.
    ///
    pub fn new_warning<S>(message: S) -> Self
    where
        S: std::fmt::Display,
    {
        Self::new_warning_with_data(None, None, message, None, None)
    }

    ///
    /// Creates a new simple error with a contract data.
    ///
    pub fn new_error_contract<S>(path: Option<&str>, message: S) -> Self
    where
        S: std::fmt::Display,
    {
        let source_location = path.map(|path| {
            SourceLocation::new(
                path.to_owned(),
                SourceLocation::UNKNOWN_OFFSET,
                SourceLocation::UNKNOWN_OFFSET,
            )
        });
        Self::new_error_with_data(path, None, message, source_location, None)
    }

    ///
    /// Creates a new error with optional code location and error code.
    ///
    pub fn new_error_with_data<S>(
        path: Option<&str>,
        error_code: Option<&str>,
        message: S,
        source_location: Option<SourceLocation>,
        sources: Option<&BTreeMap<String, InputSource>>,
    ) -> Self
    where
        S: std::fmt::Display,
    {
        Self::new(path, "Error", error_code, message, source_location, sources)
    }

    ///
    /// Creates a new warning with optional code location and error code.
    ///
    pub fn new_warning_with_data<S>(
        path: Option<&str>,
        error_code: Option<&str>,
        message: S,
        source_location: Option<SourceLocation>,
        sources: Option<&BTreeMap<String, InputSource>>,
    ) -> Self
    where
        S: std::fmt::Display,
    {
        Self::new(
            path,
            "Warning",
            error_code,
            message,
            source_location,
            sources,
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.formatted_message)
    }
}
