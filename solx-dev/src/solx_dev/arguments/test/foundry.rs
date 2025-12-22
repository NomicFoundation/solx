//!
//! `solx` Foundry test arguments.
//!

use std::path::PathBuf;

use clap::Args;

///
/// `solx` Foundry test arguments.
///
#[derive(Args)]
pub struct Foundry {
    /// Enable verbose output, e.g. to inspect extra flags.
    #[arg(short, long)]
    pub verbose: bool,

    /// Foundry test configuration path.
    #[arg(long, default_value = "./solx-dev/foundry-tests.toml")]
    pub test_config_path: PathBuf,

    /// Foundry compiler downloader configuration path.
    #[arg(long, default_value = "./solx-dev/solc-downloader.json")]
    pub downloader_config_path: PathBuf,

    /// Foundry projects temporary directory path.
    #[arg(long, default_value = "./temp-foundry-projects")]
    pub projects_dir: PathBuf,

    /// Foundry compilers temporary directory path.
    #[arg(long, default_value = "./temp-foundry-compilers")]
    pub compilers_dir: PathBuf,

    /// Solidity version to use for pragmas and other anchors.
    #[arg(long, default_value = "0.8.33")]
    pub solidity_version: String,

    /// Filter to run only projects matching the specified substring.
    #[arg(long, num_args = 1..)]
    pub project_filter: Vec<String>,

    /// Foundry output reports directory path.
    #[arg(long, default_value = "./temp-foundry-reports")]
    pub output_dir: PathBuf,
}
