//!
//! Tests for `HealthIssue` harness-degradation detection.
//!

use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::health_issue::HealthIssue;
use crate::suite_outcome::SuiteOutcome;

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
