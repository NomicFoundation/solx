//!
//! The gated-gas half of a `Changed` output verdict.
//!

///
/// The gated-gas half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub struct GasChange {
    /// Gated-gas measurements that differed between PR and `main`.
    pub diffs: u64,
    /// Gated-gas measurements compared in total.
    pub cells: u64,
    /// The suite whose gated gas changed.
    pub label: String,
}
