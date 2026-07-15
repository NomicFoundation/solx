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
use solx_benchmark_converter::SummarySuite;
use solx_benchmark_converter::ToolchainMatrix;

use self::arguments::Arguments;

///
/// Loads a suite's benchmark, returning `None` when the suite was not part of
/// this run (no flag). A flag pointing to a missing or unreadable file means
/// the suite errored before writing a valid report — rendered as a failed row
/// rather than aborting the summary for the healthy suites.
///
fn load_suite(
    label: &str,
    report_file: &str,
    path: Option<PathBuf>,
    report_url: Option<String>,
    gas_is_gate: bool,
    matrix: ToolchainMatrix,
) -> Option<SummarySuite> {
    let path = path?;
    let benchmark = match Benchmark::try_from(path.clone()) {
        Ok(benchmark) => Some(benchmark),
        Err(error) => {
            eprintln!(
                "Warning: {label} benchmark {path:?} is unusable ({error}); rendering the suite as errored."
            );
            None
        }
    };
    Some(SummarySuite {
        label: label.to_owned(),
        report_file: report_file.to_owned(),
        benchmark,
        // A skipped upload step passes its URL through as an empty string.
        report_url: report_url.filter(|url| !url.is_empty()),
        gas_is_gate,
        matrix,
    })
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let suites: Vec<SummarySuite> = [
        load_suite(
            "solx-tester",
            "solx-tester-report.xlsx",
            arguments.tester,
            arguments.tester_url,
            true,
            ToolchainMatrix::Tester,
        ),
        load_suite(
            "Foundry",
            "foundry-report.xlsx",
            arguments.foundry,
            arguments.foundry_url,
            false,
            ToolchainMatrix::Project,
        ),
        load_suite(
            "Hardhat",
            "hardhat-report.xlsx",
            arguments.hardhat,
            arguments.hardhat_url,
            false,
            ToolchainMatrix::Project,
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
