//!
//! The integration-test summary generator binary.
//!
//! Reads the per-suite merged benchmark JSONs produced by the integration
//! workflow and emits the single markdown comment it posts to the PR.
//!

pub(crate) mod arguments;

use std::path::PathBuf;

use clap::Parser;

use solx_benchmark_converter::Benchmark;
use solx_benchmark_converter::SuiteKind;
use solx_benchmark_converter::SuiteOutcome;
use solx_benchmark_converter::SummarySuite;

use self::arguments::Arguments;

///
/// Loads a suite's benchmark, returning `None` when the suite was not part of
/// this invocation (no flags at all). A skipped step outcome renders as an
/// explicit "did not run" row; a flag pointing to a missing or unreadable
/// file means the suite errored before writing a valid report — rendered as
/// a failed row rather than aborting the summary for the healthy suites.
///
fn load_suite(
    kind: SuiteKind,
    path: Option<PathBuf>,
    report_url: Option<String>,
    outcome: Option<String>,
) -> Option<SummarySuite> {
    let outcome = SuiteOutcome::from_step_outcome(outcome.as_deref());
    let benchmark = match (outcome, path) {
        (SuiteOutcome::Skipped, _) => None,
        (_, None) => return None,
        (_, Some(path)) => match Benchmark::try_from(path.clone()) {
            Ok(benchmark) => Some(benchmark),
            Err(error) => {
                eprintln!(
                    "Warning: {} benchmark {path:?} is unusable ({error}); rendering the suite as errored.",
                    kind.label()
                );
                None
            }
        },
    };
    Some(SummarySuite {
        label: kind.label().to_owned(),
        report_file: kind.report_file().to_owned(),
        benchmark,
        // A skipped upload step passes its URL through as an empty string.
        report_url: report_url.filter(|url| !url.is_empty()),
        gas_is_gate: kind.gas_is_gate(),
        matrix: kind.matrix(),
        outcome,
    })
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let suites: Vec<SummarySuite> = [
        load_suite(
            SuiteKind::Tester,
            arguments.tester,
            arguments.tester_url,
            arguments.tester_outcome,
        ),
        load_suite(
            SuiteKind::Foundry,
            arguments.foundry,
            arguments.foundry_url,
            arguments.foundry_outcome,
        ),
        load_suite(
            SuiteKind::Hardhat,
            arguments.hardhat,
            arguments.hardhat_url,
            arguments.hardhat_outcome,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();

    if suites.is_empty() {
        anyhow::bail!("No suite benchmarks were provided; nothing to summarize.");
    }

    let markdown = solx_benchmark_converter::render_summary(&suites);
    std::fs::write(arguments.output_path.as_path(), markdown).map_err(|error| {
        anyhow::anyhow!("Summary file {:?} writing: {error}", arguments.output_path)
    })?;

    Ok(())
}
