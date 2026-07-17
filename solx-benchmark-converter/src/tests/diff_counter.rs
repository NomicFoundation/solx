//!
//! Tests for DiffCounter.
//!

use crate::output::summary::diff_counter::DiffCounter;

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
