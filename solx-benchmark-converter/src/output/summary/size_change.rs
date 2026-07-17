//!
//! The size half of a `Changed` output verdict.
//!

///
/// The size half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) struct SizeChange {
    pub(crate) diffs: u64,
    pub(crate) cells: u64,
    pub(crate) delta_bytes: i128,
}
