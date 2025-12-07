//!
//! Benchmark input format.
//!

pub mod error;
pub mod foundry_gas;
pub mod foundry_size;
pub mod source;

use std::path::Path;

use crate::model::benchmark::Benchmark;

use self::error::Error as InputError;
use self::foundry_gas::FoundryGasReport;
use self::foundry_size::FoundrySizeReport;

///
/// Benchmark input format.
///
#[derive(Debug, serde::Deserialize)]
pub struct Input {
    /// The original report.
    pub data: Report,

    /// Project identifier.
    /// Must be added to the original report.
    pub project: String,
    /// Optional toolchain identifier.
    /// Can be added to the original report.
    pub toolchain: String,
}

impl Input {
    ///
    /// Creates a new benchmark input.
    ///
    pub fn new<R: Into<Report>, S1: Into<String>, S2: Into<String>>(
        report: R,
        project: S1,
        toolchain: S2,
    ) -> Self {
        Self {
            data: report.into(),
            project: project.into(),
            toolchain: toolchain.into(),
        }
    }
}

///
/// Enum representing various benchmark formats from tooling.
///
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Report {
    /// Benchmark converter's native benchmark report format.
    Native(Benchmark),
    /// Foundry gas report.
    FoundryGas(FoundryGasReport),
    /// Foundry size report.
    FoundrySize(FoundrySizeReport),
}

impl From<Benchmark> for Report {
    fn from(benchmark: Benchmark) -> Self {
        Self::Native(benchmark)
    }
}

impl From<FoundryGasReport> for Report {
    fn from(foundry_gas_report: FoundryGasReport) -> Self {
        Self::FoundryGas(foundry_gas_report)
    }
}

impl From<FoundrySizeReport> for Report {
    fn from(foundry_size_report: FoundrySizeReport) -> Self {
        Self::FoundrySize(foundry_size_report)
    }
}

impl TryFrom<&Path> for Input {
    type Error = InputError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let text = std::fs::read_to_string(path).map_err(|error| InputError::Reading {
            error,
            path: path.to_path_buf(),
        })?;
        if text.is_empty() {
            return Err(InputError::EmptyFile {
                path: path.to_path_buf(),
            });
        }
        let json: Self =
            serde_json::from_str(text.as_str()).map_err(|error| InputError::Parsing {
                error,
                path: path.to_path_buf(),
            })?;
        Ok(json)
    }
}
