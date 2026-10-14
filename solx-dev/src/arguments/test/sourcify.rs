//!
//! `solx` Sourcify sweep arguments.
//!

use std::path::PathBuf;

use clap::Args;

///
/// `solx` Sourcify sweep arguments.
///
#[derive(Args)]
pub struct Sourcify {
    //
    // Configuration Paths
    //
    /// Sourcify sweep configuration path.
    #[arg(
        long,
        default_value = "./solx-dev/sourcify-tests.toml",
        help_heading = "Configuration Paths"
    )]
    pub test_config_path: PathBuf,

    /// Compiler downloader configuration path.
    #[arg(
        long,
        default_value = "./solx-compiler-downloader/dev-compilers.json",
        help_heading = "Configuration Paths"
    )]
    pub downloader_config_path: PathBuf,

    //
    // Directories
    //
    /// Directory the pinned corpus is downloaded to and its slice extracted in.
    #[arg(
        long,
        default_value = "./temp-sourcify-corpus",
        help_heading = "Directories"
    )]
    pub corpus_dir: PathBuf,

    /// Local corpus to sweep instead of the pinned one: a corpus root with `contracts/`, or a
    /// contracts directory such as `./tests/sourcify-sweep/fixtures`.
    #[arg(long, help_heading = "Directories")]
    pub corpus_path: Option<PathBuf>,

    /// Results path, one JSON line per contract.
    #[arg(
        long,
        default_value = "./temp-sourcify-reports/results.jsonl",
        help_heading = "Directories"
    )]
    pub output_path: PathBuf,

    //
    // Test Filtering
    //
    /// Solidity version every version pragma is rewritten to, as a lower bound.
    #[arg(long, default_value = "0.8.34", help_heading = "Test Filtering")]
    pub solidity_version: String,

    /// Number of round-robin shards the sorted corpus is split into.
    #[arg(long, default_value_t = 1, help_heading = "Test Filtering")]
    pub shard_count: usize,

    /// Shard to sweep, in `[0, --shard-count)`.
    #[arg(long, default_value_t = 0, help_heading = "Test Filtering")]
    pub shard_index: usize,

    /// Stop after this many contracts.
    #[arg(long, help_heading = "Test Filtering")]
    pub limit: Option<usize>,

    //
    // Limits
    //
    /// Number of concurrent compiler processes [default: available parallelism].
    #[arg(long, help_heading = "Limits")]
    pub threads: Option<usize>,

    /// Per-compilation timeout in seconds.
    #[arg(long, default_value_t = 300, help_heading = "Limits")]
    pub timeout: u64,

    /// Address-space cap of each compiler process in MiB, so an out-of-memory compilation aborts
    /// instead of taking the machine down; 0 for none.
    #[arg(long, default_value_t = 0, help_heading = "Limits")]
    pub memory_limit_mib: u64,
}

///
/// `solx` Sourcify sweep report arguments.
///
#[derive(Args)]
pub struct SourcifyReport {
    /// Results paths written by `test sourcify`, e.g. one per shard.
    #[arg(long, num_args = 1.., required = true)]
    pub results_paths: Vec<PathBuf>,

    /// Markdown report path [default: standard output].
    #[arg(long)]
    pub output_path: Option<PathBuf>,

    /// Report heading.
    #[arg(long, default_value = "Sourcify sweep")]
    pub title: String,
}
