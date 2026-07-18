//!
//! How the suite's workflow step ended.
//!

use clap::ValueEnum;

///
/// How the suite's workflow step ended, distinguishing a suite that never ran
/// from one that errored, and qualifying data written by a step that then
/// failed. The `Default` exists only so `SuiteStats` can derive one;
/// `from_suite` always sets the real outcome over it.
///
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum SuiteOutcome {
    /// The step ran to completion.
    #[default]
    Success,
    /// The step ran but exited nonzero, or was cancelled; any report it wrote
    /// may be partial.
    #[value(alias = "cancelled")]
    Failure,
    /// The step never ran after an earlier hard failure; not the suite's fault.
    Skipped,
}
