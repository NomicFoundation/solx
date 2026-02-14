//!
//! Benchmark analyzer output.
//!

pub mod comparison;
pub mod format;
pub mod json;
pub mod xlsx;

use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::output::comparison::Comparison;
use crate::output::format::Format;
use crate::output::json::Json;
use crate::output::xlsx::Xlsx;

///
/// Result of comparing two benchmarks.
///
pub enum Output {
    /// Benchmark JSON output.
    Json(String),
    /// Benchmark Excel/XLSX output.
    Xlsx(rust_xlsxwriter::Workbook),
}

impl Output {
    ///
    /// Writes the benchmark results to a file using a provided serializer.
    ///
    pub fn write_to_file(self, path: PathBuf) -> anyhow::Result<()> {
        match self {
            Output::Json(content) => {
                std::fs::write(path.as_path(), content)
                    .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} writing: {error}"))?;
            }
            Output::Xlsx(mut workbook) => {
                workbook
                    .save(path.as_path())
                    .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} writing: {error}"))?;
            }
        }
        Ok(())
    }
}

impl TryFrom<(Benchmark, Vec<Comparison>, Format)> for Output {
    type Error = anyhow::Error;

    fn try_from(
        (benchmark, comparisons, output_format): (Benchmark, Vec<Comparison>, Format),
    ) -> Result<Self, Self::Error> {
        Ok(match output_format {
            Format::Json => Json::from(benchmark).into(),
            Format::Xlsx => Xlsx::try_from((benchmark, comparisons))?.into(),
        })
    }
}

impl From<Json> for Output {
    fn from(value: Json) -> Self {
        Output::Json(value.content)
    }
}

impl From<Xlsx> for Output {
    fn from(value: Xlsx) -> Self {
        Output::Xlsx(value.finalize())
    }
}
