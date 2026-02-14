//!
//! Benchmark input format.
//!

pub mod build_failures;
pub mod compilation_time;
pub mod error;
pub mod foundry_gas;
pub mod foundry_size;
pub mod test_failures;
pub mod testing_time;

use std::path::Path;

use crate::benchmark::Benchmark;

use self::build_failures::BuildFailuresReport;
use self::compilation_time::CompilationTimeReport;
use self::error::Error as InputError;
use self::foundry_gas::FoundryGasReport;
use self::foundry_size::FoundrySizeReport;
use self::test_failures::TestFailuresReport;
use self::testing_time::TestingTimeReport;

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
    /// Compilation time report.
    CompilationTime(CompilationTimeReport),
    /// Testing time report.
    TestingTime(TestingTimeReport),
    /// Build failures report.
    BuildFailures(BuildFailuresReport),
    /// Test failures report.
    TestFailures(TestFailuresReport),
}

impl From<Benchmark> for Report {
    fn from(report: Benchmark) -> Self {
        Self::Native(report)
    }
}

impl From<FoundryGasReport> for Report {
    fn from(report: FoundryGasReport) -> Self {
        Self::FoundryGas(report)
    }
}

impl From<FoundrySizeReport> for Report {
    fn from(report: FoundrySizeReport) -> Self {
        Self::FoundrySize(report)
    }
}

impl From<CompilationTimeReport> for Report {
    fn from(report: CompilationTimeReport) -> Self {
        Self::CompilationTime(report)
    }
}

impl From<TestingTimeReport> for Report {
    fn from(report: TestingTimeReport) -> Self {
        Self::TestingTime(report)
    }
}

impl From<BuildFailuresReport> for Report {
    fn from(report: BuildFailuresReport) -> Self {
        Self::BuildFailures(report)
    }
}

impl From<TestFailuresReport> for Report {
    fn from(report: TestFailuresReport) -> Self {
        Self::TestFailures(report)
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
