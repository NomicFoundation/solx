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
use std::path::PathBuf;

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

    ///
    /// Resolves the converter's input paths: a single directory expands to
    /// every JSON file underneath it; explicit file paths pass through
    /// unchanged — the workflow's no-baseline fallback passes exactly one file.
    ///
    pub fn resolve_paths(paths: Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
        if paths.len() == 1 && paths[0].is_dir() {
            let resolution_pattern = format!("{}/**/*.json", paths[0].to_string_lossy());
            return Ok(glob::glob(resolution_pattern.as_str())?
                .filter_map(Result::ok)
                .collect());
        }
        if paths.is_empty() {
            anyhow::bail!("No input files provided. Use `--input-paths` to specify input files.");
        }
        Ok(paths)
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Input;

    /// A unique writable directory per test; tests must not share contents.
    fn scratch_dir(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "solx-benchmark-converter-{test}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(dir.as_path());
        std::fs::create_dir_all(dir.as_path()).expect("scratch directory creation");
        dir
    }

    #[test]
    fn a_single_file_is_an_input_not_a_directory() {
        let dir = scratch_dir("single-file");
        let file = dir.join("candidate.json");
        std::fs::write(file.as_path(), "{}").expect("file writing");
        assert_eq!(
            Input::resolve_paths(vec![file.clone()]).expect("resolution"),
            [file]
        );
    }

    #[test]
    fn a_single_directory_expands_to_its_json_files() {
        let dir = scratch_dir("directory");
        std::fs::create_dir_all(dir.join("nested")).expect("nested directory creation");
        for name in ["a.json", "nested/b.json", "ignored.txt"] {
            std::fs::write(dir.join(name), "{}").expect("file writing");
        }
        let mut resolved = Input::resolve_paths(vec![dir.clone()]).expect("resolution");
        resolved.sort();
        assert_eq!(resolved, [dir.join("a.json"), dir.join("nested/b.json")]);
    }

    #[test]
    fn no_inputs_is_an_error() {
        assert!(Input::resolve_paths(Vec::new()).is_err());
    }
}
