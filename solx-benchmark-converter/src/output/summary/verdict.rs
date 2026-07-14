//!
//! The decision layer of the integration summary.
//!
//! Pure functions reduce the per-suite statistics to typed verdicts — the
//! complete decision table behind the comment's headline lines, testable
//! without parsing markdown. Turning a verdict into prose is the rendering
//! layer's concern; nothing here formats beyond carrying labels.
//!

use super::stats::SuiteStats;

///
/// Whether the PR preserved compiler output, judged over every suite's size
/// comparisons plus the gas comparisons of gating suites. Non-gated gas
/// (fuzz-noisy Foundry/Hardhat runs) never influences this verdict.
///
#[derive(Debug, PartialEq)]
pub(crate) enum OutputVerdict {
    /// No size or gated-gas comparisons were collected — never a green
    /// checkmark over empty data.
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
    /// The suite's benchmark data matched no recognized toolchain naming.
    UnrecognizedToolchains { label: String },
    /// PR runs with no `main` counterpart; their failures are not compared.
    Unbaselined {
        label: String,
        runs: usize,
        failures: usize,
    },
}

///
/// The output-invariance verdict over all suites.
///
pub(crate) fn output_verdict(stats: &[SuiteStats]) -> OutputVerdict {
    let size_cells: u64 = stats.iter().map(|s| s.size.cells).sum();
    let size_diffs: u64 = stats.iter().map(|s| s.size.diffs).sum();
    let size_delta: i128 = stats.iter().map(|s| s.size.delta).sum();
    let gated: Vec<&SuiteStats> = stats.iter().filter(|s| s.gas_is_gate).collect();
    let gated_gas_cells: u64 = gated.iter().map(|s| s.gas.cells).sum();
    let gated_gas_diffs: u64 = gated.iter().map(|s| s.gas.diffs).sum();
    let gas_label = gated
        .iter()
        .filter(|s| s.gas.collected())
        .map(|s| s.label.as_str())
        .collect::<Vec<_>>()
        .join(" / ");

    if size_diffs == 0 && gated_gas_diffs == 0 {
        if size_cells == 0 && gated_gas_cells == 0 {
            return OutputVerdict::NoData;
        }
        return OutputVerdict::Preserving {
            size_cells,
            gated_gas_cells,
            gas_label,
        };
    }
    OutputVerdict::Changed {
        size: (size_diffs > 0).then_some(SizeChange {
            diffs: size_diffs,
            cells: size_cells,
            delta_bytes: size_delta,
        }),
        gas: (gated_gas_diffs > 0).then_some(GasChange {
            diffs: gated_gas_diffs,
            cells: gated_gas_cells,
            label: gas_label,
        }),
    }
}

///
/// The failure-regression verdict over all suites.
///
pub(crate) fn failure_verdict(stats: &[SuiteStats]) -> FailureVerdict {
    if stats.iter().all(|s| s.new_failures() == 0) {
        FailureVerdict::Clean {
            pre_existing: stats
                .iter()
                .filter(|s| s.baseline_failures() > 0)
                .map(|s| (s.label.clone(), s.baseline_failures()))
                .collect(),
        }
    } else {
        FailureVerdict::Regressed {
            suites: stats
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

///
/// Every harness-degradation signal, in rendering order: errored suites,
/// unrecognized naming, then unbaselined runs.
///
pub(crate) fn health_issues(stats: &[SuiteStats]) -> Vec<HealthIssue> {
    let mut issues = Vec::new();
    for s in stats.iter().filter(|s| !s.available) {
        issues.push(HealthIssue::SuiteErrored {
            label: s.label.clone(),
        });
    }
    for s in stats.iter().filter(|s| s.classification_failed()) {
        issues.push(HealthIssue::UnrecognizedToolchains {
            label: s.label.clone(),
        });
    }
    for s in stats.iter().filter(|s| s.unbaselined_runs > 0) {
        issues.push(HealthIssue::Unbaselined {
            label: s.label.clone(),
            runs: s.unbaselined_runs,
            failures: s.unbaselined_failures,
        });
    }
    issues
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
        assert_eq!(output_verdict(&[]), OutputVerdict::NoData);
        assert_eq!(
            output_verdict(&[available("Foundry")]),
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
            output_verdict(&[foundry]),
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
            output_verdict(&[tester]),
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
        let OutputVerdict::Changed { size, gas } = output_verdict(&[tester]) else {
            panic!("expected Changed");
        };
        assert_eq!(size, None);
        assert!(gas.is_some());
    }

    #[test]
    fn clean_failures_carry_the_pre_existing_counts() {
        let foundry = SuiteStats {
            baseline_test_failures: 5,
            ..available("Foundry")
        };
        assert_eq!(
            failure_verdict(&[foundry, available("Hardhat")]),
            FailureVerdict::Clean {
                pre_existing: vec![("Foundry".to_owned(), 5)],
            }
        );
    }

    #[test]
    fn regressed_lists_only_the_regressed_suites() {
        let foundry = SuiteStats {
            new_build_failures: 1,
            new_test_failures: 2,
            baseline_test_failures: 5,
            ..available("Foundry")
        };
        assert_eq!(
            failure_verdict(&[foundry, available("Hardhat")]),
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
        assert_eq!(
            health_issues(&[errored, drifted, unbaselined]),
            vec![
                HealthIssue::SuiteErrored {
                    label: "solx-tester".to_owned(),
                },
                HealthIssue::UnrecognizedToolchains {
                    label: "Foundry".to_owned(),
                },
                HealthIssue::Unbaselined {
                    label: "Hardhat".to_owned(),
                    runs: 1,
                    failures: 5,
                },
            ]
        );
    }
}
