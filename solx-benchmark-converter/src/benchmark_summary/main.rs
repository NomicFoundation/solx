//!
//! The integration-test summary generator binary.
//!
//! Reads the per-suite merged benchmark JSONs produced by the integration
//! workflow and emits the single markdown comment it posts to the PR.
//!

pub(crate) mod arguments;

use clap::Parser;

use solx_benchmark_converter::SuiteKind;
use solx_benchmark_converter::Summary;
use solx_benchmark_converter::SummarySuite;

use self::arguments::Arguments;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let suites: Vec<SummarySuite> = [
        SuiteKind::Tester.load(
            arguments.tester,
            arguments.tester_url,
            arguments.tester_outcome,
        ),
        SuiteKind::Foundry.load(
            arguments.foundry,
            arguments.foundry_url,
            arguments.foundry_outcome,
        ),
        SuiteKind::Hardhat.load(
            arguments.hardhat,
            arguments.hardhat_url,
            arguments.hardhat_outcome,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();

    let summary = Summary::new(suites);
    if summary.is_empty() {
        anyhow::bail!("No suite benchmarks were provided; nothing to summarize.");
    }

    std::fs::write(arguments.output_path.as_path(), summary.render()).map_err(|error| {
        anyhow::anyhow!("Summary file {:?} writing: {error}", arguments.output_path)
    })?;

    Ok(())
}
