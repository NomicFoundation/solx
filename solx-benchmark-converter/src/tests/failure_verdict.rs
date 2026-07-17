//!
//! Tests for the failure-regression verdict.
//!

use crate::output::summary::suite_failures::SuiteFailures;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::failure_verdict::FailureVerdict;

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
