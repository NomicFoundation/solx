//!
//! Crate-internal test suite.
//!

mod benchmark;
mod diff_counter;
mod failure_regressions;
mod failure_verdict;
mod health_issue;
mod input;
mod output_verdict;
mod summary;
mod toolchain_matrix;
mod top_movers;
mod utils;
mod xlsx;

use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::suite_stats::SuiteStats;

impl SuiteStats {
    /// An available suite carrying only the given label, for the verdict tests.
    fn available(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            available: true,
            ..Default::default()
        }
    }
}

impl DiffCounter {
    /// A counter with the given tallies, for the output-verdict tests.
    fn counted(cells: u64, diffs: u64, delta: i128) -> Self {
        Self {
            cells,
            diffs,
            delta,
        }
    }
}
