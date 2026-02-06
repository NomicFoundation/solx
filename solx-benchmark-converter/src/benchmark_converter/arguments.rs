//!
//! The benchmark analyzer arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// The benchmark analyzer arguments.
///
#[derive(Debug, Parser)]
#[command(about, long_about = None, arg_required_else_help = true)]
pub struct Arguments {
    /// Suppresses the terminal output.
    #[arg(short, long)]
    pub quiet: bool,

    /// Input files.
    /// If only one path is provided, it is treated as a directory with JSON files.
    pub input_paths: Vec<PathBuf>,

    /// Benchmark output format: `json`, `csv`, or `json-lnt`.
    #[arg(long = "output-format", alias = "benchmark-format", default_value_t = solx_benchmark_converter::OutputFormat::Xlsx)]
    pub output_format: solx_benchmark_converter::OutputFormat,

    /// Output files.
    #[arg(long)]
    pub output_path: PathBuf,
}
