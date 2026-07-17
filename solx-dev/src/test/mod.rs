//!
//! `solx` test tools.
//!

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
