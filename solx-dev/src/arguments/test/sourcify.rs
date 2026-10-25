//!
//! `solx` Sourcify sweep arguments.
//!

use std::num::NonZeroUsize;
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

    /// Local corpus to sweep instead of the pinned one, with a `contracts/` directory, such as
    /// `./tests/sourcify-sweep/fixtures`.
    #[arg(long, help_heading = "Directories")]
    pub local_corpus_dir: Option<PathBuf>,

    //
    // Output
    //
    /// Results path, one JSON line per contract.
    #[arg(
        long,
        default_value = "./temp-sourcify-reports/results.jsonl",
        help_heading = "Output"
    )]
    pub output_path: PathBuf,

    //
    // Test Filtering
    //
    /// Solidity version every version pragma is rewritten to, as a lower bound.
    #[arg(long, default_value = "0.8.34", help_heading = "Test Filtering")]
    pub solidity_version: String,

    /// Number of round-robin shards the sorted corpus is split into.
    #[arg(long, default_value_t = NonZeroUsize::MIN, help_heading = "Test Filtering")]
    pub shard_count: NonZeroUsize,

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

    /// Resident memory cap of each compiler with its workers in MiB, so an out-of-memory
    /// compilation is killed instead of taking the machine down.
    #[arg(long, help_heading = "Limits")]
    pub memory_limit_mib: Option<u64>,
}

///
/// `solx` Sourcify sweep report arguments.
///
#[derive(Args)]
pub struct SourcifyReport {
    /// Results path written by `test sourcify`, or several shards' results concatenated.
    #[arg(long)]
    pub results_path: PathBuf,

    /// Markdown report path [default: standard output].
    #[arg(long)]
    pub output_path: Option<PathBuf>,

    /// Report heading.
    #[arg(long, default_value = "Sourcify sweep")]
    pub title: String,
}
