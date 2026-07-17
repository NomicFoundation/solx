//!
//! Tests for the inline "new failures" regression listing.
//!

use crate::output::summary::failure_regressions::FailureRegression;
use crate::output::summary::failure_regressions::FailureRegressions;

#[test]
fn failure_regressions_rank_by_magnitude() {
    let mut regressions = FailureRegressions::default();
    for (label, main, pr) in [("small", 1, 2), ("worst", 0, 9), ("middle", 2, 5)] {
        regressions.push(FailureRegression {
            label: label.to_owned(),
            mode: "legacy".to_owned(),
            kind: "test",
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
