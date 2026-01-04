//!
//! Benchmark analyzer output.
//!

pub mod file;
pub mod format;
pub mod json;
pub mod xlsx;

use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::input::source::Source;
use crate::output::format::Format;
use crate::output::json::Json;
use crate::output::xlsx::Xlsx;

use self::file::File;

///
/// Result of comparing two benchmarks.
///
pub enum Output {
    /// Benchmark output is a single unnamed file.
    SingleFile(String),
    /// Benchmark output is a single unnamed file.
    SingleFileXlsx(rust_xlsxwriter::Workbook),
    /// Benchmark output is structured as a file tree, relative to some
    /// user-provided output directory.
    MultipleFiles(Vec<File>),
}

impl Output {
    ///
    /// Writes the benchmark results to a file using a provided serializer.
    ///
    pub fn write_to_file(self, path: PathBuf) -> anyhow::Result<()> {
        match self {
            Output::SingleFile(content) => {
                std::fs::write(path.as_path(), content)
                    .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} writing: {error}"))?;
            }
            Output::SingleFileXlsx(mut workbook) => {
                workbook
                    .save(path.as_path())
                    .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} writing: {error}"))?;
            }
            Output::MultipleFiles(files) => {
                if !files.is_empty() {
                    std::fs::create_dir_all(&path)?;
                }
                for File {
                    path: relative_path,
                    content: contents,
                } in files
                {
                    let file_path = path.join(relative_path);
                    std::fs::write(file_path.as_path(), contents).map_err(|error| {
                        anyhow::anyhow!("Benchmark file {file_path:?} writing: {error}")
                    })?;
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<(Benchmark, Source, Format)> for Output {
    type Error = anyhow::Error;

    fn try_from(
        (benchmark, input_source, output_format): (Benchmark, Source, Format),
    ) -> Result<Self, Self::Error> {
        Ok(match output_format {
            Format::Json => Json::from(benchmark).into(),
            Format::Xlsx => Xlsx::try_from((benchmark, input_source))?.into(),
        })
    }
}

impl From<Json> for Output {
    fn from(value: Json) -> Self {
        Output::SingleFile(value.content)
    }
}

impl From<Xlsx> for Output {
    fn from(value: Xlsx) -> Self {
        Output::SingleFileXlsx(value.finalize())
    }
}
