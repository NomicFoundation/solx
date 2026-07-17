//!
//! Whether any suite failed more than its `main` baseline.
//!

use crate::output::summary::suite_failures::SuiteFailures;
use crate::output::summary::suite_stats::SuiteStats;
use crate::utils::agreeing;
use crate::utils::commas;

///
/// Whether any suite failed more than its `main` baseline.
///
#[derive(Debug, PartialEq)]
pub enum FailureVerdict {
    /// No suite paired a PR run with a `main` counterpart — never a green
    /// checkmark over zero comparisons.
    NoData,
    /// No suite regressed; failures already present on `main` are carried
    /// per suite label so the verdict can say so.
    Clean { pre_existing: Vec<(String, usize)> },
    /// At least one suite regressed.
    Regressed { suites: Vec<SuiteFailures> },
}

impl FailureVerdict {
    ///
    /// The failure-regression verdict, over the suites that actually compared
    /// something — errored, empty, and unclassifiable suites carry no
    /// PR-vs-main pairs and must not feed a green line.
    ///
    pub fn from_stats(stats: &[SuiteStats]) -> Self {
        let compared: Vec<&SuiteStats> = stats
            .iter()
            .filter(|s| s.available && !s.classification_failed())
            .collect();
        if compared.iter().all(|s| s.paired_runs == 0) {
            return Self::NoData;
        }
        if compared.iter().all(|s| s.new_failures() == 0) {
            Self::Clean {
                pre_existing: compared
                    .iter()
                    .filter(|s| s.baseline_failures() > 0)
                    .map(|s| (s.label.clone(), s.baseline_failures()))
                    .collect(),
            }
        } else {
            Self::Regressed {
                suites: compared
                    .iter()
                    .filter(|s| s.new_failures() > 0)
                    .map(|s| SuiteFailures {
                        label: s.label.clone(),
                        new_build: s.new_build_failures,
                        new_test: s.new_test_failures,
                    })
                    .collect(),
            }
        }
    }

    /// The failure-regression verdict line.
    pub fn line(self) -> String {
        match self {
            Self::NoData => {
                "⚪ **No failure data** — no PR run had a `main` counterpart to compare against."
                    .to_owned()
            }
            Self::Clean { pre_existing } if pre_existing.is_empty() => {
                "✅ **No new failures**.".to_owned()
            }
            Self::Clean { pre_existing } => {
                let pre: Vec<String> = pre_existing
                    .iter()
                    .map(|(label, count)| format!("{label}'s {}", commas(*count as u64)))
                    .collect();
                format!(
                    "✅ **No new failures** — {} {} already present on `main`.",
                    pre.join(" / "),
                    agreeing(
                        pre_existing.iter().map(|(_, count)| *count as u64).sum(),
                        "failure",
                        "failures"
                    )
                )
            }
            Self::Regressed { suites } => {
                let parts: Vec<String> = suites
                    .iter()
                    .map(|suite| {
                        format!(
                            "{}: {}",
                            suite.label,
                            SuiteFailures::kinds(suite.new_build, suite.new_test)
                        )
                    })
                    .collect();
                format!("❌ **New failures** — {}.", parts.join("; "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::output::summary::failure_verdict::*;

    #[test]
    fn clean_failures_carry_the_pre_existing_counts() {
        let foundry = SuiteStats {
            paired_runs: 1,
            baseline_test_failures: 5,
            ..SuiteStats::available("Foundry")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[foundry, SuiteStats::available("Hardhat")]),
            FailureVerdict::Clean {
                pre_existing: vec![("Foundry".to_owned(), 5)],
            }
        );
    }

    #[test]
    fn regressed_lists_only_the_regressed_suites() {
        let foundry = SuiteStats {
            paired_runs: 1,
            new_build_failures: 1,
            new_test_failures: 2,
            baseline_test_failures: 5,
            ..SuiteStats::available("Foundry")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[foundry, SuiteStats::available("Hardhat")]),
            FailureVerdict::Regressed {
                suites: vec![SuiteFailures {
                    label: "Foundry".to_owned(),
                    new_build: 1,
                    new_test: 2,
                }],
            }
        );
    }

    #[test]
    fn failure_verdict_is_no_data_when_nothing_compared() {
        let errored = SuiteStats {
            label: "solx-tester".to_owned(),
            available: false,
            ..Default::default()
        };
        let drifted = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 0,
            baseline_test_failures: 40,
            ..SuiteStats::available("Foundry")
        };
        let unbaselined = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            unbaselined_runs: 1,
            ..SuiteStats::available("Hardhat")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[errored, drifted, unbaselined]),
            FailureVerdict::NoData
        );
    }
}
