//!
//! A degradation of the harness itself — the comment must never look green
//! while the data underneath it is missing or unreadable.
//!

use crate::output::summary::suite_stats::SuiteStats;
use crate::suite_outcome::SuiteOutcome;

///
/// A degradation of the harness itself — the comment must never look green
/// while the data underneath it is missing or unreadable.
///
#[derive(Debug, PartialEq)]
pub enum HealthIssue {
    /// The suite ran but produced no usable report.
    SuiteErrored { label: String },
    /// The suite's step failed after its report was written — the data is
    /// real but possibly incomplete, so the green cells need a caveat.
    StepFailed { label: String },
    /// The report parsed but recorded no runs — the suite tested nothing.
    EmptySuite { label: String },
    /// The suite's benchmark data matched no recognized toolchain naming.
    UnrecognizedToolchains { label: String },
    /// Individual runs matching no declared toolchain name, in a suite whose
    /// PR data is otherwise present — e.g. a renamed or foreign baseline.
    UnrecognizedRuns { label: String, modes: Vec<String> },
    /// Recognized runs whose mode carries no recognized pipeline token —
    /// e.g. a new codegen letter the tables don't know yet.
    UnrecognizedPipelines { label: String, modes: Vec<String> },
    /// PR runs with no `main` counterpart; their failures are not compared.
    Unbaselined {
        label: String,
        runs: usize,
        failures: usize,
    },
    /// Main runs with no PR counterpart — the comparison set shrank.
    MainOnly {
        label: String,
        runs: usize,
        failures: usize,
    },
}

impl HealthIssue {
    ///
    /// Every harness-degradation signal, in rendering order: errored suites,
    /// unrecognized naming, then unbaselined runs.
    ///
    pub fn from_stats(stats: &[SuiteStats]) -> Vec<Self> {
        let mut issues = Vec::new();
        for s in stats
            .iter()
            .filter(|s| !s.available && s.outcome != SuiteOutcome::Skipped)
        {
            issues.push(Self::SuiteErrored {
                label: s.label.clone(),
            });
        }
        for s in stats
            .iter()
            .filter(|s| s.available && s.outcome == SuiteOutcome::Failure)
        {
            issues.push(Self::StepFailed {
                label: s.label.clone(),
            });
        }
        for s in stats.iter().filter(|s| s.is_empty_report()) {
            issues.push(Self::EmptySuite {
                label: s.label.clone(),
            });
        }
        for s in stats.iter().filter(|s| s.classification_failed()) {
            issues.push(Self::UnrecognizedToolchains {
                label: s.label.clone(),
            });
        }
        for s in stats
            .iter()
            .filter(|s| !s.classification_failed() && !s.unrecognized_modes.is_empty())
        {
            issues.push(Self::UnrecognizedRuns {
                label: s.label.clone(),
                modes: s.unrecognized_modes.iter().cloned().collect(),
            });
        }
        for s in stats
            .iter()
            .filter(|s| !s.unrecognized_pipelines.is_empty())
        {
            issues.push(Self::UnrecognizedPipelines {
                label: s.label.clone(),
                modes: s.unrecognized_pipelines.iter().cloned().collect(),
            });
        }
        for s in stats.iter().filter(|s| s.unbaselined_runs > 0) {
            issues.push(Self::Unbaselined {
                label: s.label.clone(),
                runs: s.unbaselined_runs,
                failures: s.unbaselined_failures,
            });
        }
        for s in stats.iter().filter(|s| s.main_orphan_runs > 0) {
            issues.push(Self::MainOnly {
                label: s.label.clone(),
                runs: s.main_orphan_runs,
                failures: s.main_orphan_failures,
            });
        }
        issues
    }
}

#[cfg(test)]
mod tests {
    use crate::output::summary::health_issue::*;

    #[test]
    fn health_issues_cover_every_degradation() {
        let errored = SuiteStats {
            label: "solx-tester".to_owned(),
            available: false,
            ..Default::default()
        };
        let drifted = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 0,
            ..SuiteStats::available("Foundry")
        };
        let unbaselined = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            unbaselined_runs: 1,
            unbaselined_failures: 5,
            ..SuiteStats::available("Hardhat")
        };
        let foreign_run = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 1,
            unrecognized_modes: ["04.mason-legacy".to_owned()].into(),
            ..SuiteStats::available("Foundry 2")
        };
        let empty = SuiteStats::available("Hardhat 2");
        let shrunken = SuiteStats {
            total_runs: 3,
            pr_runs_seen: 1,
            paired_runs: 1,
            main_orphan_runs: 1,
            main_orphan_failures: 7,
            ..SuiteStats::available("Foundry 3")
        };
        let step_failed = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            outcome: SuiteOutcome::Failure,
            ..SuiteStats::available("Hardhat 3")
        };
        let unknown_pipeline = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 1,
            unrecognized_pipelines: ["01.solx-solx-L-M3B3-0.8.34".to_owned()].into(),
            ..SuiteStats::available("Foundry 4")
        };
        assert_eq!(
            HealthIssue::from_stats(&[
                errored,
                drifted,
                unbaselined,
                foreign_run,
                empty,
                shrunken,
                step_failed,
                unknown_pipeline
            ]),
            vec![
                HealthIssue::SuiteErrored {
                    label: "solx-tester".to_owned(),
                },
                HealthIssue::StepFailed {
                    label: "Hardhat 3".to_owned(),
                },
                HealthIssue::EmptySuite {
                    label: "Hardhat 2".to_owned(),
                },
                HealthIssue::UnrecognizedToolchains {
                    label: "Foundry".to_owned(),
                },
                HealthIssue::UnrecognizedRuns {
                    label: "Foundry 2".to_owned(),
                    modes: vec!["04.mason-legacy".to_owned()],
                },
                HealthIssue::UnrecognizedPipelines {
                    label: "Foundry 4".to_owned(),
                    modes: vec!["01.solx-solx-L-M3B3-0.8.34".to_owned()],
                },
                HealthIssue::Unbaselined {
                    label: "Hardhat".to_owned(),
                    runs: 1,
                    failures: 5,
                },
                HealthIssue::MainOnly {
                    label: "Foundry 3".to_owned(),
                    runs: 1,
                    failures: 7,
                },
            ]
        );
    }
}
