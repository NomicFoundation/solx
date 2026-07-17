//!
//! Tests for top movers.
//!

use crate::output::summary::top_movers::TopMovers;

#[test]
fn movers_rank_by_magnitude_regardless_of_direction() {
    let mut movers = TopMovers::default();
    movers.push("small", "legacy", 100, 103);
    movers.push("shrunk", "legacy", 100, 80);
    movers.push("grown", "legacy", 100, 110);
    let ranked = movers.ranked();
    let labels: Vec<&str> = ranked
        .iter()
        .map(|movement| movement.label.as_str())
        .collect();
    assert_eq!(labels, ["shrunk", "grown", "small"]);
}
