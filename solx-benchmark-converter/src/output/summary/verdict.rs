//!
//! The decision layer of the integration summary.
//!
//! Each verdict is constructed from the per-suite statistics behind the
//! comment's headline lines. Turning a verdict into prose is the rendering
//! layer's concern; nothing here formats beyond carrying labels.
//!

use super::SuiteOutcome;
use super::stats::DiffCounter;
use super::stats::SuiteStats;

///
/// Whether the PR preserved compiler output, judged over every suite's size
/// comparisons plus the gas comparisons of gating suites. Non-gated gas
/// (fuzz-noisy Foundry/Hardhat runs) never influences this verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) enum OutputVerdict {
    /// No size or gated-gas comparison paired a PR value with a `main` one,
    /// whether nothing was collected or everything collected was one-sided —
    /// never a green checkmark over data that was never compared.
    NoData,
    /// Every collected comparison is identical.
    Preserving {
        size_cells: u64,
        gated_gas_cells: u64,
        /// Labels of the gated suites with gas data, e.g. "solx-tester".
        gas_label: String,
    },
    /// At least one comparison differs; each differing signal is present.
    Changed {
        size: Option<SizeChange>,
        gas: Option<GasChange>,
    },
}

///
/// The size half of a `Changed` verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) struct SizeChange {
    pub(crate) diffs: u64,
    pub(crate) cells: u64,
    pub(crate) delta_bytes: i128,
}

///
/// The gated-gas half of a `Changed` verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) struct GasChange {
    pub(crate) diffs: u64,
    pub(crate) cells: u64,
    pub(crate) label: String,
}

///
/// Whether any suite failed more than its `main` baseline.
///
#[derive(Debug, PartialEq)]
pub(crate) enum FailureVerdict {
    /// No suite paired a PR run with a `main` counterpart — never a green
    /// checkmark over zero comparisons.
    NoData,
    /// No suite regressed; failures already present on `main` are carried
    /// per suite label so the verdict can say so.
    Clean { pre_existing: Vec<(String, usize)> },
    /// At least one suite regressed.
    Regressed { suites: Vec<SuiteFailures> },
}

///
/// One regressed suite's new failures by kind.
///
#[derive(Debug, PartialEq)]
pub(crate) struct SuiteFailures {
    pub(crate) label: String,
    pub(crate) new_build: usize,
    pub(crate) new_test: usize,
}

///
/// A degradation of the harness itself — the comment must never look green
/// while the data underneath it is missing or unreadable.
///
#[derive(Debug, PartialEq)]
pub(crate) enum HealthIssue {
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

impl OutputVerdict {
    ///
    /// The output-invariance verdict over all suites.
    ///
    pub(crate) fn from_stats(stats: &[SuiteStats]) -> Self {
        let mut size = DiffCounter::default();
        let mut gas = DiffCounter::default();
        let mut gas_labels = Vec::new();
        for s in stats {
            size.absorb(&s.size);
            if s.gas_is_gate {
                gas.absorb(&s.gas);
                if s.gas.collected() {
                    gas_labels.push(s.label.as_str());
                }
            }
        }
        let gas_label = gas_labels.join(" / ");

        if size.diffs == 0 && gas.diffs == 0 {
            if size.cells == 0 && gas.cells == 0 {
                return Self::NoData;
            }
            return Self::Preserving {
                size_cells: size.cells,
                gated_gas_cells: gas.cells,
                gas_label,
            };
        }
        Self::Changed {
            size: (size.diffs > 0).then_some(SizeChange {
                diffs: size.diffs,
                cells: size.cells,
                delta_bytes: size.delta,
            }),
            gas: (gas.diffs > 0).then_some(GasChange {
                diffs: gas.diffs,
                cells: gas.cells,
                label: gas_label,
            }),
        }
    }
}

impl FailureVerdict {
    ///
    /// The failure-regression verdict, over the suites that actually compared
    /// something — errored, empty, and unclassifiable suites carry no
    /// PR-vs-main pairs and must not feed a green line.
    ///
    pub(crate) fn from_stats(stats: &[SuiteStats]) -> Self {
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
}

impl HealthIssue {
    ///
    /// Every harness-degradation signal, in rendering order: errored suites,
    /// unrecognized naming, then unbaselined runs.
    ///
    pub(crate) fn from_stats(stats: &[SuiteStats]) -> Vec<Self> {
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
    use super::*;
    use crate::output::summary::stats::DiffCounter;

    fn counted(cells: u64, diffs: u64, delta: i128) -> DiffCounter {
        DiffCounter {
            cells,
            diffs,
            delta,
        }
    }

    fn available(label: &str) -> SuiteStats {
        SuiteStats {
            label: label.to_owned(),
            available: true,
            ..Default::default()
        }
    }

    #[test]
    fn no_data_over_empty_comparisons() {
        assert_eq!(OutputVerdict::from_stats(&[]), OutputVerdict::NoData);
        assert_eq!(
            OutputVerdict::from_stats(&[available("Foundry")]),
            OutputVerdict::NoData
        );
    }

    #[test]
    fn ungated_gas_jitter_does_not_break_preserving() {
        let foundry = SuiteStats {
            gas_is_gate: false,
            size: counted(4, 0, 0),
            gas: counted(10, 5, 123),
            ..available("Foundry")
        };
        assert_eq!(
            OutputVerdict::from_stats(&[foundry]),
            OutputVerdict::Preserving {
                size_cells: 4,
                gated_gas_cells: 0,
                gas_label: String::new(),
            }
        );
    }

    #[test]
    fn changed_carries_each_differing_signal() {
        let tester = SuiteStats {
            gas_is_gate: true,
            size: counted(5, 2, -42),
            gas: counted(9, 1, 3),
            ..available("solx-tester")
        };
        assert_eq!(
            OutputVerdict::from_stats(&[tester]),
            OutputVerdict::Changed {
                size: Some(SizeChange {
                    diffs: 2,
                    cells: 5,
                    delta_bytes: -42,
                }),
                gas: Some(GasChange {
                    diffs: 1,
                    cells: 9,
                    label: "solx-tester".to_owned(),
                }),
            }
        );
    }

    #[test]
    fn gated_gas_diff_alone_changes_the_verdict() {
        let tester = SuiteStats {
            gas_is_gate: true,
            size: counted(5, 0, 0),
            gas: counted(9, 1, 3),
            ..available("solx-tester")
        };
        let OutputVerdict::Changed { size, gas } = OutputVerdict::from_stats(&[tester]) else {
            panic!("expected Changed");
        };
        assert_eq!(size, None);
        assert!(gas.is_some());
    }

    #[test]
    fn clean_failures_carry_the_pre_existing_counts() {
        let foundry = SuiteStats {
            paired_runs: 1,
            baseline_test_failures: 5,
            ..available("Foundry")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[foundry, available("Hardhat")]),
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
            ..available("Foundry")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[foundry, available("Hardhat")]),
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
            ..available("Foundry")
        };
        let unbaselined = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            unbaselined_runs: 1,
            ..available("Hardhat")
        };
        assert_eq!(
            FailureVerdict::from_stats(&[errored, drifted, unbaselined]),
            FailureVerdict::NoData
        );
    }

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
            ..available("Foundry")
        };
        let unbaselined = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            unbaselined_runs: 1,
            unbaselined_failures: 5,
            ..available("Hardhat")
        };
        let foreign_run = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 1,
            unrecognized_modes: ["04.mason-legacy".to_owned()].into(),
            ..available("Foundry 2")
        };
        let empty = available("Hardhat 2");
        let shrunken = SuiteStats {
            total_runs: 3,
            pr_runs_seen: 1,
            paired_runs: 1,
            main_orphan_runs: 1,
            main_orphan_failures: 7,
            ..available("Foundry 3")
        };
        let step_failed = SuiteStats {
            total_runs: 1,
            pr_runs_seen: 1,
            outcome: SuiteOutcome::Failure,
            ..available("Hardhat 3")
        };
        let unknown_pipeline = SuiteStats {
            total_runs: 2,
            pr_runs_seen: 1,
            unrecognized_pipelines: ["01.solx-solx-L-M3B3-0.8.34".to_owned()].into(),
            ..available("Foundry 4")
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
