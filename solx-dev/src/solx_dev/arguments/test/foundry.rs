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

    //
    // Configuration Paths
    //
    /// Foundry test configuration path.
    #[arg(
        long,
        default_value = "./solx-dev/foundry-tests.toml",
        help_heading = "Configuration Paths"
    )]
    pub test_config_path: PathBuf,

    /// Foundry compiler downloader configuration path.
    #[arg(
        long,
        default_value = "./solx-compiler-downloader/dev-compilers.json",
        help_heading = "Configuration Paths"
    )]
    pub downloader_config_path: PathBuf,

    //
    // Directories
    //
    /// Foundry projects temporary directory path.
    #[arg(
        long,
        default_value = "./temp-foundry-projects",
        help_heading = "Directories"
    )]
    pub projects_dir: PathBuf,

    /// Foundry compilers temporary directory path.
    #[arg(
        long,
        default_value = "./temp-foundry-compilers",
        help_heading = "Directories"
    )]
    pub compilers_dir: PathBuf,

    /// Foundry output reports directory path.
    #[arg(
        long,
        default_value = "./temp-foundry-reports",
        help_heading = "Directories"
    )]
    pub output_dir: PathBuf,

    //
    // Test Filtering
    //
    /// Solidity version to use for pragmas and other anchors.
    #[arg(long, default_value = "0.8.33", help_heading = "Test Filtering")]
    pub solidity_version: String,

    /// Filter to run only projects matching the specified substring.
    #[arg(long, num_args = 1.., help_heading = "Test Filtering")]
    pub project_filter: Vec<String>,
}
