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

impl std::str::FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "xlsx" => Ok(Self::Xlsx),
            string => anyhow::bail!(
                "Unknown benchmark format `{string}`. Supported formats: {}",
                vec![Self::Json, Self::Xlsx]
                    .into_iter()
                    .map(|element| element.to_string().to_lowercase())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Json => write!(f, "json"),
            Format::Xlsx => write!(f, "xlsx"),
        }
    }
}
