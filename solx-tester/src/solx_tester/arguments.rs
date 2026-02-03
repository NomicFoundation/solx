//!
//! `solx` tester arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// `solx` tester arguments.
///
#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct Arguments {
    /// The logging level.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppresses the terminal output.
    #[arg(short, long)]
    pub quiet: bool,

    /// Saves all IRs produced by compilers to `./debug/` directory.
    #[arg(short = 'D', long)]
    pub debug: bool,

    /// Prints the REVM trace to standard output.
    #[arg(long)]
    pub trace: bool,

    /// Runs tests only in modes that contain any string from the specified ones.
    #[arg(short, long)]
    pub mode: Vec<String>,

    /// Runs only tests whose name contains any string from the specified ones.
    #[arg(short, long)]
    pub path: Vec<String>,

    /// Runs only tests from the specified groups.
    #[structopt(short, long)]
    pub group: Vec<String>,

    /// The benchmark output path, if requested.
    #[structopt(short, long)]
    pub benchmark: Option<PathBuf>,

    /// The benchmark output format: `json`, `csv`, or `json-lnt`.
    /// Using `json-lnt` requires providing the path to a JSON file describing the
    /// benchmarking context via `--benchmark-context`.
    #[structopt(long = "benchmark-format", default_value_t = solx_benchmark_converter::OutputFormat::Json)]
    pub benchmark_format: solx_benchmark_converter::OutputFormat,

    /// Sets the number of threads, which execute the tests concurrently.
    #[structopt(short, long)]
    pub threads: Option<usize>,

    /// Path to the `solx` executable.
    /// Is set to `solx` by default.
    #[structopt(long)]
    pub solx: Option<PathBuf>,

    /// Specify the compiler toolchain.
    /// Available arguments: `solx`, `solc`, `solx-mlir`.
    /// Is set to `solx` by default.
    #[structopt(long)]
    pub toolchain: Option<solx_tester::Toolchain>,

    /// Choose between `build` to compile tests only without running, and `run` to compile and run.
    #[structopt(long, default_value_t = solx_tester::Workflow::BuildAndRun)]
    pub workflow: solx_tester::Workflow,

    /// Path to the default `solc` executables download configuration file.
    #[structopt(long)]
    pub solc_bin_config_path: Option<PathBuf>,

    /// Sets the `verify each` option in LLVM.
    #[structopt(long)]
    pub llvm_verify_each: bool,

    /// Sets the `debug logging` option in LLVM.
    #[structopt(long)]
    pub llvm_debug_logging: bool,
}
