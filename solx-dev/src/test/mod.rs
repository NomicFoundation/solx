//!
//! `solx` test tools.
//!

use colored::Colorize;
use itertools::Itertools;

pub mod foundry;
pub mod hardhat;
pub mod solx_tester;

/// Codegen variants every enabled compiler is tested with.
pub(crate) const CODEGENS: [&str; 2] = ["legacy", "viaIR"];

///
/// The toolchain identifier that benchmark inputs, correctness tables, and
/// report comparisons are keyed by.
///
pub(crate) fn toolchain_name(compiler_name: &str, codegen: &str) -> String {
    format!("{compiler_name}-{codegen}")
}

///
/// Writes a suite's benchmark JSON and XLSX report into the output directory
/// — one emission tail shared by the Foundry and Hardhat runners so the two
/// cannot drift.
///
pub(crate) fn write_reports(
    benchmark: solx_benchmark_converter::Benchmark,
    comparisons: Vec<solx_benchmark_converter::OutputComparison>,
    output_directory: std::path::PathBuf,
    kind: solx_benchmark_converter::SuiteKind,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_directory.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "{} {} output reports directory {output_directory:?}: {error}",
            "Creating".bright_green().bold(),
            kind.label(),
        )
    })?;
    let base_path = crate::utils::absolute_path(output_directory)?;

    solx_benchmark_converter::Output::from(solx_benchmark_converter::OutputJson::from(&benchmark))
        .write_to_file(base_path.join(kind.benchmark_file()))?;

    let output: solx_benchmark_converter::Output = (
        benchmark,
        comparisons,
        solx_benchmark_converter::OutputFormat::Xlsx,
    )
        .try_into()?;
    let mut output_path = base_path;
    output_path.push(kind.report_file());
    eprintln!(
        "{} the spreadsheet report to {}",
        solx_utils::cargo_status_ok("Writing"),
        output_path.to_string_lossy().bright_white().bold()
    );
    output.write_to_file(output_path)?;
    Ok(())
}

///
/// Errors if any attempted project x enabled toolchain pair produced no
/// benchmark data.
///
/// Structurally unreachable today — every runner iteration either records an
/// input or aborts the run — this is a tripwire for future control-flow edits
/// that skip a pair without recording it, the silent-omission class behind
/// #497.
///
pub(crate) fn verify_benchmark_coverage(
    benchmark_inputs: &[solx_benchmark_converter::Input],
    projects: &[String],
    compiler_names: &[&str],
) -> anyhow::Result<()> {
    for project in projects {
        for (compiler_name, codegen) in compiler_names.iter().cartesian_product(CODEGENS) {
            let toolchain_name = toolchain_name(compiler_name, codegen);
            if !benchmark_inputs
                .iter()
                .any(|input| input.project == *project && input.toolchain == toolchain_name)
            {
                anyhow::bail!(
                    "Harness self-check failed: project {project} with toolchain {toolchain_name} produced no benchmark data",
                );
            }
        }
    }
    Ok(())
}
