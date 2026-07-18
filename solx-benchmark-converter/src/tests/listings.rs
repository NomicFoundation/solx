//!
//! Tests for the ranked inline listings: top movers and failure regressions.
//!

use crate::output::summary::failure_kind::FailureKind;
use crate::output::summary::failure_regressions::FailureRegression;
use crate::output::summary::failure_regressions::FailureRegressions;
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

#[test]
fn failure_regressions_rank_by_magnitude() {
    let mut regressions = FailureRegressions::default();
    for (label, main, pr) in [("small", 1, 2), ("worst", 0, 9), ("middle", 2, 5)] {
        regressions.push(FailureRegression {
            label: label.to_owned(),
            mode: "legacy".to_owned(),
            kind: FailureKind::Test,
            main,
            pr,
        });
    }
    let labels: Vec<&str> = regressions
        .ranked()
        .iter()
        .map(|regression| regression.label.as_str())
        .collect();
    assert_eq!(labels, ["worst", "middle", "small"]);
}
