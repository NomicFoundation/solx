//!
//! The gated-gas half of a `Changed` output verdict.
//!

///
/// The gated-gas half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub struct GasChange {
    pub diffs: u64,
    pub cells: u64,
    pub label: String,
}
