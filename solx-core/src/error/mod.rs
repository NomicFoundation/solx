//!
//! Compilation error.
//!

pub mod stack_too_deep;

use self::stack_too_deep::StackTooDeep;

///
/// Compilation error.
///
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum Error {
    /// The stack-too-deep error.
    #[error("{0}")]
    StackTooDeep(StackTooDeep),
    /// Standard JSON error.
    #[error("{0}")]
    StandardJson(solx_standard_json::OutputError),
    /// Generic error.
    #[error("{0}")]
    Generic(String),
}

impl Error {
    ///
    /// A shortcut constructor for a `StackTooDeep` error.
    ///
    pub fn stack_too_deep(spill_area_size: u64, is_size_fallback: bool) -> Self {
        Error::StackTooDeep(StackTooDeep {
            spill_area_size,
            is_size_fallback,
        })
    }

    ///
    /// Converts the error into a standard JSON output error.
    ///
    /// Non-standard-JSON variants (e.g. worker deaths on LLVM fatal errors) are wrapped
    /// into an error attributed to the contract at `path`.
    ///
    pub fn into_standard_json(self, path: Option<&str>) -> solx_standard_json::OutputError {
        match self {
            Error::StandardJson(error) => error,
            error => solx_standard_json::OutputError::new_error_contract(path, error),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::Generic(error.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Generic(error.to_string())
    }
}

impl From<solx_standard_json::OutputError> for Error {
    fn from(error: solx_standard_json::OutputError) -> Self {
        Error::StandardJson(error)
    }
}
