//!
//! The integration-test summary generator arguments.
//!

use std::path::PathBuf;

use clap::Parser;

///
/// Generates the single PR summary comment from the per-suite benchmark JSONs.
///
/// Each suite is optional: a suite whose flag is not passed is omitted from
/// the comment, while a flag pointing to a missing or unreadable file renders
/// that suite as errored.
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
    /// The solx-tester step outcome (success/failure/skipped); a skipped
    /// suite renders as an explicit "did not run" row.
    #[arg(long)]
    pub tester_outcome: Option<String>,

    /// Merged Foundry benchmark JSON (gas is fuzz-noisy → excluded from gate).
    #[arg(long)]
    pub foundry: Option<PathBuf>,
    /// Artifact download URL for the Foundry XLSX report.
    #[arg(long)]
    pub foundry_url: Option<String>,
    /// The Foundry step outcome (success/failure/skipped).
    #[arg(long)]
    pub foundry_outcome: Option<String>,

    /// Merged Hardhat benchmark JSON (gas is fuzz-noisy → excluded from gate).
    #[arg(long)]
    pub hardhat: Option<PathBuf>,
    /// Artifact download URL for the Hardhat XLSX report.
    #[arg(long)]
    pub hardhat_url: Option<String>,
    /// The Hardhat step outcome (success/failure/skipped).
    #[arg(long)]
    pub hardhat_outcome: Option<String>,

    /// Output markdown file.
    #[arg(long)]
    pub output_path: PathBuf,
}
