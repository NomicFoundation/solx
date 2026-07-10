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

use self::arguments::Arguments;

///
/// Loads a suite's benchmark, returning `None` when the file is absent.
///
fn load_suite(
    label: &str,
    path: Option<PathBuf>,
    report_url: Option<String>,
    gas_is_gate: bool,
) -> anyhow::Result<Option<SummarySuite>> {
    // No flag at all: the suite was not part of this run.
    let Some(path) = path else {
        return Ok(None);
    };
    // Flag given but no file: the suite was expected but errored before writing
    // its report. Surface it as a failed row rather than dropping it.
    let benchmark = if path.exists() {
        Some(Benchmark::try_from(path)?)
    } else {
        eprintln!("Warning: {label} benchmark {path:?} is absent; the suite errored.");
        None
    };
    Ok(Some(SummarySuite {
        label: label.to_owned(),
        benchmark,
        // A skipped upload step passes its URL through as an empty string.
        report_url: report_url.filter(|url| !url.is_empty()),
        gas_is_gate,
    }))
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let suites: Vec<SummarySuite> = [
        load_suite("solx-tester", arguments.tester, arguments.tester_url, true)?,
        load_suite("Foundry", arguments.foundry, arguments.foundry_url, false)?,
        load_suite("Hardhat", arguments.hardhat, arguments.hardhat_url, false)?,
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
