//!
//! Output benchmark format.
//!

use clap::ValueEnum;

///
/// Output benchmark format.
///
#[derive(Debug, Default, Clone, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Unstable JSON format, corresponds to the inner data model of benchmark converter.
    #[default]
    Json,
    /// Excel spreadsheet format.
    Xlsx,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Json => write!(f, "json"),
            Format::Xlsx => write!(f, "xlsx"),
        }
    }
}
