//!
//! Regressions collected for the inline "new failures" listing.
//!

use super::failure_regression::FailureRegression;

///
/// Regressions collected for the inline "new failures" listing.
///
#[derive(Default)]
pub(crate) struct FailureRegressions(Vec<FailureRegression>);

impl FailureRegressions {
    pub(crate) fn push(&mut self, regression: FailureRegression) {
        self.0.push(regression);
    }

    /// The regressions ordered by descending magnitude, so the renderer lists
    /// the worst first and counts the rest as "+N more".
    pub(crate) fn ranked(&self) -> Vec<&FailureRegression> {
        let mut regressions: Vec<&FailureRegression> = self.0.iter().collect();
        regressions.sort_by_key(|regression| std::cmp::Reverse(regression.pr - regression.main));
        regressions
    }
}

#[cfg(test)]
mod tests {
    use super::FailureRegression;
    use super::FailureRegressions;

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
}
