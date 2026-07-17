//!
//! The size half of a `Changed` output verdict.
//!

///
/// The size half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub struct SizeChange {
    pub diffs: u64,
    pub cells: u64,
    pub delta_bytes: i128,
}
