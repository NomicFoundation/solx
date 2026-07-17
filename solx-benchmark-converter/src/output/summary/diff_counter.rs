//!
//! Counts PR-vs-main comparison pairs and the differing subset.
//!

///
/// Counts PR-vs-main comparison pairs and the differing subset.
///
#[derive(Default)]
pub(crate) struct DiffCounter {
    /// Pairs where at least one side produced a value.
    pub(crate) cells: u64,
    /// Pairs whose sides differ.
    pub(crate) diffs: u64,
    /// Signed PR-minus-main total over the differing pairs.
    pub(crate) delta: i128,
}

impl DiffCounter {
    ///
    /// Records one pair, ignoring pairs where neither side produced a value.
    /// Returns whether the recorded pair differs.
    ///
    pub(crate) fn observe(&mut self, pr: u64, main: u64) -> bool {
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
    pub(crate) fn collected(&self) -> bool {
        self.cells > 0
    }

    /// Folds another counter in, for cross-suite aggregate verdicts.
    pub(crate) fn absorb(&mut self, other: &Self) {
        self.cells += other.cells;
        self.diffs += other.diffs;
        self.delta += other.delta;
    }

    /// A counter with the given tallies, for the output-verdict tests.
    #[cfg(test)]
    pub(crate) fn counted(cells: u64, diffs: u64, delta: i128) -> Self {
        Self {
            cells,
            diffs,
            delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiffCounter;

    #[test]
    fn diff_counter_skips_uncollected_pairs_and_sums_deltas() {
        let mut counter = DiffCounter::default();
        assert!(!counter.observe(0, 0));
        assert!(!counter.collected());
        assert!(!counter.observe(100, 100));
        assert!(counter.observe(90, 100));
        assert!(counter.observe(115, 100));
        assert!(counter.collected());
        assert_eq!(counter.cells, 3);
        assert_eq!(counter.diffs, 2);
        assert_eq!(counter.delta, 5);
    }
}
