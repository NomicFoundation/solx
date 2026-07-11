//!
//! `solx` test tools.
//!

pub mod foundry;
pub mod hardhat;
pub mod solx_tester;

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
    toolchains: &[String],
) -> anyhow::Result<()> {
    for project in projects {
        for toolchain in toolchains {
            if !benchmark_inputs
                .iter()
                .any(|input| input.project == *project && input.toolchain == *toolchain)
            {
                anyhow::bail!(
                    "Harness self-check failed: project {project} with toolchain {toolchain} produced no benchmark data",
                );
            }
        }
    }
    Ok(())
}
