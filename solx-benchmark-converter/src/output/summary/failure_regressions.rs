//!
//! Regressions collected for the inline "new failures" listing.
//!

use std::cmp::Reverse;

use crate::output::summary::failure_kind::FailureKind;

///
/// Regressions collected for the inline "new failures" listing.
///
#[derive(Default)]
pub struct FailureRegressions(Vec<FailureRegression>);

///
/// One row whose PR run failed more than its main counterpart.
///
pub struct FailureRegression {
    /// The row label: the project or case that regressed.
    pub label: String,
    /// The humanized toolchain the regression was measured on.
    pub mode: String,
    /// Whether the excess failures are build or test failures.
    pub kind: FailureKind,
    /// The `main` counterpart's failure count.
    pub main: usize,
    /// The PR run's failure count.
    pub pr: usize,
}

impl FailureRegressions {
    /// Records a regression for the inline listing.
    pub fn push(&mut self, regression: FailureRegression) {
        self.0.push(regression);
    }

    /// The regressions ordered by descending magnitude, so the renderer lists
    /// the worst first and counts the rest as "+N more".
    pub fn ranked(&self) -> Vec<&FailureRegression> {
        let mut regressions: Vec<&FailureRegression> = self.0.iter().collect();
        regressions.sort_by_key(|regression| Reverse(regression.pr - regression.main));
        regressions
    }
}
