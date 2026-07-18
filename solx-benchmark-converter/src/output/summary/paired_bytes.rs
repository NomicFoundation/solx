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
    /// The PR toolchain's summed bytecode.
    pub pr: u64,
    /// The baseline toolchain's summed bytecode.
    pub baseline: u64,
}
