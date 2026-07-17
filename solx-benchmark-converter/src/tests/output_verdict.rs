//!
//! Tests for the output-invariance verdict.
//!

use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::summary_template::output_verdict::OutputVerdict;
use crate::output::summary::summary_template::output_verdict::gas_change::GasChange;
use crate::output::summary::summary_template::output_verdict::size_change::SizeChange;

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
