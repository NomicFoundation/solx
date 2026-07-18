//!
//! Tests for the summary verdict types: the diff counter, output-invariance
//! and failure-regression verdicts, and harness-health detection.
//!

use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::suite_failures::SuiteFailures;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::failure_verdict::FailureVerdict;
use crate::output::summary::summary_template::health_issue::HealthIssue;
use crate::output::summary::summary_template::output_verdict::OutputVerdict;
use crate::output::summary::summary_template::output_verdict::gas_change::GasChange;
use crate::output::summary::summary_template::output_verdict::size_change::SizeChange;
use crate::suite_outcome::SuiteOutcome;

#[test]
fn diff_counter_skips_uncollected_pairs_and_sums_deltas() {
    let mut counter = DiffCounter::default();
    assert!(!counter.observe(0, 0));
    assert!(!counter.collected());
    assert!(!counter.observe(100, 100));
    assert!(counter.observe(90, 100));
    assert!(counter.observe(115, 100));
    assert!(counter.collected());
    assert_eq!(counter.cells, 3);
    assert_eq!(counter.diffs, 2);
    assert_eq!(counter.delta, 5);
}

#[test]
fn no_data_over_empty_comparisons() {
    assert_eq!(OutputVerdict::from_stats(&[]), OutputVerdict::NoData);
    assert_eq!(
        OutputVerdict::from_stats(&[SuiteStats::available("Foundry")]),
        OutputVerdict::NoData
    );
}

#[test]
fn ungated_gas_jitter_does_not_break_preserving() {
    let foundry = SuiteStats {
        gas_is_gate: false,
        size: DiffCounter::counted(4, 0, 0),
        gas: DiffCounter::counted(10, 5, 123),
        ..SuiteStats::available("Foundry")
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
        size: DiffCounter::counted(5, 2, -42),
        gas: DiffCounter::counted(9, 1, 3),
        ..SuiteStats::available("solx-tester")
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
        size: DiffCounter::counted(5, 0, 0),
        gas: DiffCounter::counted(9, 1, 3),
        ..SuiteStats::available("solx-tester")
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
