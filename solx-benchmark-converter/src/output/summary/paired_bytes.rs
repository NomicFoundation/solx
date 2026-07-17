//!
//! Bytecode totals summed over contracts that both the PR and the baseline
//! toolchain emitted.
//!

///
/// Bytecode totals summed over contracts that both the PR and the baseline
/// toolchain emitted.
///
#[derive(Default)]
pub struct PairedBytes {
    pub pr: u64,
    pub baseline: u64,
}
