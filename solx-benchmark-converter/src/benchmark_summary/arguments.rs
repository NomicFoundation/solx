//!
//! The integration-test summary generator arguments.
//!

use std::path::PathBuf;

use clap::Parser;

use solx_benchmark_converter::SuiteOutcome;

///
/// Generates the single PR summary comment from the per-suite benchmark JSONs.
///
/// Every suite reports its step outcome; a skipped one renders as an explicit
/// "did not run" row. Its benchmark JSON is optional: absent, the suite is
/// omitted; present but unreadable, the suite renders as errored.
///
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct Arguments {
    /// Merged solx-tester benchmark JSON; gas is deterministic here, so it is gated.
    #[arg(long)]
    pub tester: Option<PathBuf>,
    /// Artifact download URL for the solx-tester XLSX report.
    #[arg(long)]
    pub tester_url: Option<String>,
    /// The solx-tester step outcome; a skipped suite renders as an explicit
    /// "did not run" row.
    #[arg(long)]
    pub tester_outcome: SuiteOutcome,

    /// Merged Foundry benchmark JSON; gas is fuzz-noisy, so excluded from gate.
    #[arg(long)]
    pub foundry: Option<PathBuf>,
    /// Artifact download URL for the Foundry XLSX report.
    #[arg(long)]
    pub foundry_url: Option<String>,
    /// The Foundry step outcome.
    #[arg(long)]
    pub foundry_outcome: SuiteOutcome,

    /// Merged Hardhat benchmark JSON; gas is fuzz-noisy, so excluded from gate.
    #[arg(long)]
    pub hardhat: Option<PathBuf>,
    /// Artifact download URL for the Hardhat XLSX report.
    #[arg(long)]
    pub hardhat_url: Option<String>,
    /// The Hardhat step outcome.
    #[arg(long)]
    pub hardhat_outcome: SuiteOutcome,

    /// Output markdown file.
    #[arg(long)]
    pub output_path: PathBuf,
}
