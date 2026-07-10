//!
//! The integration-test summary generator arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// Generates the single PR summary comment from the per-suite benchmark JSONs.
///
/// Each suite is optional: a missing benchmark file (the suite did not run, or
/// produced no data) is simply omitted from the comment.
///
#[derive(Debug, Parser)]
#[command(about, long_about = None, arg_required_else_help = true)]
pub struct Arguments {
    /// Merged solx-tester benchmark JSON (gas is deterministic here → gated).
    #[arg(long)]
    pub tester: Option<PathBuf>,
    /// Artifact download URL for the solx-tester XLSX report.
    #[arg(long)]
    pub tester_url: Option<String>,

    /// Merged Foundry benchmark JSON (gas is fuzz-noisy → excluded from gate).
    #[arg(long)]
    pub foundry: Option<PathBuf>,
    /// Artifact download URL for the Foundry XLSX report.
    #[arg(long)]
    pub foundry_url: Option<String>,

    /// Merged Hardhat benchmark JSON (gas is fuzz-noisy → excluded from gate).
    #[arg(long)]
    pub hardhat: Option<PathBuf>,
    /// Artifact download URL for the Hardhat XLSX report.
    #[arg(long)]
    pub hardhat_url: Option<String>,

    /// Output markdown file.
    #[arg(long)]
    pub output_path: PathBuf,
}
