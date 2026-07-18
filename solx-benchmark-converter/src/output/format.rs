//!
//! Output benchmark format.
//!

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

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

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Xlsx => write!(f, "xlsx"),
        }
    }
}
