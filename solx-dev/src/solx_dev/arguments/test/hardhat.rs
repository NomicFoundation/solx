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

    //
    // Configuration Paths
    //
    /// Hardhat test configuration path.
    #[arg(
        long,
        default_value = "./solx-dev/hardhat-tests.toml",
        help_heading = "Configuration Paths"
    )]
    pub test_config_path: PathBuf,

    /// Hardhat compiler downloader configuration path.
    #[arg(
        long,
        default_value = "./solx-compiler-downloader/dev-compilers.json",
        help_heading = "Configuration Paths"
    )]
    pub downloader_config_path: PathBuf,

    //
    // Directories
    //
    /// Hardhat projects temporary directory path.
    #[arg(
        long,
        default_value = "./temp-hardhat-projects",
        help_heading = "Directories"
    )]
    pub projects_dir: PathBuf,

    /// Hardhat compilers temporary directory path.
    #[arg(
        long,
        default_value = "./temp-hardhat-compilers",
        help_heading = "Directories"
    )]
    pub compilers_dir: PathBuf,

    /// Hardhat output reports directory path.
    #[arg(
        long,
        default_value = "./temp-hardhat-reports",
        help_heading = "Directories"
    )]
    pub output_dir: PathBuf,

    //
    // Test Filtering
    //
    /// Solidity version to use for pragmas and other anchors.
    #[arg(long, help_heading = "Test Filtering")]
    pub solidity_version: Option<String>,

    /// Filter to run only projects matching the specified substring.
    #[arg(long, num_args = 1.., help_heading = "Test Filtering")]
    pub project_filter: Vec<String>,
}
