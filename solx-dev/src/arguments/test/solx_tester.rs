//!
//! `solx-tester` arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// `solx-tester` arguments passed through to the solx-tester binary.
///
#[derive(Parser)]
#[command(about = "Integration testing framework for solx", long_about = None)]
pub struct SolxTester {
    /// Enable verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress terminal output.
    #[arg(short, long)]
    pub quiet: bool,

    /// Save all IRs produced by compilers to `./debug/` directory.
    #[arg(short = 'D', long)]
    pub debug: bool,

    /// Print the REVM trace to standard output.
    #[arg(long)]
    pub trace: bool,

    /// Filter by codegen: only run tests using the Yul IR pipeline.
    #[arg(long)]
    pub via_ir: bool,

    /// Filter by optimizer settings pattern (e.g., "M3B3", "M^B3").
    #[arg(short = 'O', long)]
    pub optimizer: Option<String>,

    /// Run only tests whose path contains the specified strings.
    #[arg(short, long)]
    pub path: Vec<String>,

    /// Run only tests from the specified groups.
    #[arg(short, long)]
    pub group: Vec<String>,

    /// Benchmark output path.
    #[arg(short, long)]
    pub benchmark: Option<PathBuf>,

    /// Benchmark output format: `json` or `xlsx`.
    #[arg(long = "benchmark-format")]
    pub benchmark_format: Option<String>,

    /// Number of threads for concurrent test execution.
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Path to the Solidity compiler executable (`solx` or `solc`).
    #[arg(long)]
    pub solidity_compiler: Option<PathBuf>,

    /// Workflow: `build` (compile only) or `run` (compile and run).
    #[arg(long)]
    pub workflow: Option<String>,

    /// Path to the default `solc` executables download configuration file.
    #[arg(long)]
    pub solc_bin_config_path: Option<PathBuf>,

    /// Set the `verify each` option in LLVM.
    #[arg(long)]
    pub llvm_verify_each: bool,

    /// Set the `debug logging` option in LLVM.
    #[arg(long)]
    pub llvm_debug_logging: bool,

    /// Path to the solx-tester binary (only used by solx-dev wrapper, ignored when running solx-tester directly).
    /// Honors CARGO_TARGET_DIR if set.
    #[arg(long, hide = true)]
    pub binary: Option<PathBuf>,
}
