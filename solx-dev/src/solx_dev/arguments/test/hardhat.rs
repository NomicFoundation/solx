//!
//! `solx` Hardhat test arguments.
//!

use std::path::PathBuf;

use clap::Args;

///
/// `solx` Hardhat test arguments.
///
#[derive(Args)]
pub struct Hardhat {
    /// Enable verbose output, e.g. to inspect extra flags.
    #[arg(short, long)]
    pub verbose: bool,

    /// Hardhat test configuration path.
    #[arg(long, default_value = "./solx-dev/hardhat-tests.toml")]
    pub test_config_path: PathBuf,

    /// Hardhat compiler downloader configuration path.
    #[arg(long, default_value = "./solx-dev/solc-downloader.json")]
    pub downloader_config_path: PathBuf,

    /// Hardhat projects temporary directory path.
    #[arg(long, default_value = "./temp-hardhat-projects")]
    pub projects_dir: PathBuf,

    /// Hardhat compilers temporary directory path.
    #[arg(long, default_value = "./temp-hardhat-compilers")]
    pub compilers_dir: PathBuf,

    /// Solidity version to use for pragmas and other anchors.
    #[arg(long, default_value = "0.8.30")]
    pub solidity_version: String,

    /// Filter to run only projects matching the specified substring.
    #[arg(long, num_args = 1..)]
    pub project_filter: Vec<String>,

    /// Hardhat output reports directory path.
    #[arg(long, default_value = "./temp-hardhat-reports")]
    pub output_dir: PathBuf,
}
