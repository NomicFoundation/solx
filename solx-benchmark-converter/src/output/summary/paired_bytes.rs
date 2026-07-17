//!
//! Bytecode totals summed over contracts that both the PR and the baseline
//! toolchain emitted.
//!

///
/// Bytecode totals summed over contracts that both the PR and the baseline
/// toolchain emitted.
///
#[derive(Default)]
pub(crate) struct PairedBytes {
    pub(crate) pr: u64,
    pub(crate) baseline: u64,
}
