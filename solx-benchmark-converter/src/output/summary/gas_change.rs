//!
//! The gated-gas half of a `Changed` output verdict.
//!

///
/// The gated-gas half of a `Changed` output verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) struct GasChange {
    pub(crate) diffs: u64,
    pub(crate) cells: u64,
    pub(crate) label: String,
}
