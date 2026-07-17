//!
//! Counts PR-vs-main comparison pairs and the differing subset.
//!

use crate::utils::commas;
use crate::utils::signed_commas;

///
/// Counts PR-vs-main comparison pairs and the differing subset.
///
#[derive(Default)]
pub struct DiffCounter {
    /// Pairs where at least one side produced a value.
    pub cells: u64,
    /// Pairs whose sides differ.
    pub diffs: u64,
    /// Signed PR-minus-main total over the differing pairs.
    pub delta: i128,
}

impl DiffCounter {
    ///
    /// Records one pair, ignoring pairs where neither side produced a value.
    /// Returns whether the recorded pair differs.
    ///
    pub fn observe(&mut self, pr: u64, main: u64) -> bool {
        if pr == 0 && main == 0 {
            return false;
        }
        self.cells += 1;
        if pr == main {
            return false;
        }
        self.diffs += 1;
        self.delta += pr as i128 - main as i128;
        true
    }

    /// Whether any pair was recorded — false renders as "not collected".
    pub fn collected(&self) -> bool {
        self.cells > 0
    }

    /// Folds another counter in, for cross-suite aggregate verdicts.
    pub fn absorb(&mut self, other: &Self) {
        self.cells += other.cells;
        self.diffs += other.diffs;
        self.delta += other.delta;
    }

    /// One comparison column's table cell; the byte delta rides along for
    /// size cells.
    pub fn cell(&self, delta_suffix: bool) -> String {
        if !self.collected() {
            return "⚪ not collected".to_owned();
        }
        if self.diffs == 0 {
            return format!("✅ 0 of {}", commas(self.cells));
        }
        let delta = if delta_suffix {
            format!(" ({} B)", signed_commas(self.delta))
        } else {
            String::new()
        };
        format!("⚠️ {} of {}{delta}", commas(self.diffs), commas(self.cells))
    }
}
