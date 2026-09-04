//!
//! The size half of a `Changed` output verdict.
//!

///
/// The size half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub struct SizeChange {
    /// Contracts whose size differed between PR and `main-solc`.
    pub diffs: u64,
    /// Contracts compared in total.
    pub cells: u64,
    /// Net PR-minus-`main-solc` byte delta over the compared contracts.
    pub delta_bytes: i128,
}
