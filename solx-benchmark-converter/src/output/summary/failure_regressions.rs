//!
//! Regressions collected for the inline "new failures" listing.
//!

///
/// Regressions collected for the inline "new failures" listing.
///
#[derive(Default)]
pub struct FailureRegressions(Vec<FailureRegression>);

impl FailureRegressions {
    pub fn push(&mut self, regression: FailureRegression) {
        self.0.push(regression);
    }

    /// The regressions ordered by descending magnitude, so the renderer lists
    /// the worst first and counts the rest as "+N more".
    pub fn ranked(&self) -> Vec<&FailureRegression> {
        let mut regressions: Vec<&FailureRegression> = self.0.iter().collect();
        regressions.sort_by_key(|regression| std::cmp::Reverse(regression.pr - regression.main));
        regressions
    }
}

///
/// One row whose PR run failed more than its main counterpart.
///
pub struct FailureRegression {
    pub label: String,
    pub mode: String,
    pub kind: &'static str,
    pub main: usize,
    pub pr: usize,
}

#[cfg(test)]
mod tests {
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
}
