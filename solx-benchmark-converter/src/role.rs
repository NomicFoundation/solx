//!
//! The role a toolchain plays in the comparison.
//!

///
/// The role a toolchain plays in the comparison.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// The current commit under test.
    Pr,
    /// The `main`-branch build the PR is compared against.
    Main,
    /// The latest released solx, a full-matrix baseline.
    Latest,
    /// Upstream solc, a full-matrix baseline.
    Solc,
    /// Unrecognized naming, surfaced as a harness error, never dropped.
    Other,
}
