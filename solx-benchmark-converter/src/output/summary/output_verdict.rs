//!
//! Whether the PR preserved compiler output, judged over every suite's size
//! comparisons plus the gas comparisons of gating suites.
//!

use super::diff_counter::DiffCounter;
use super::gas_change::GasChange;
use super::size_change::SizeChange;
use super::suite_stats::SuiteStats;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::summary::suite_stats::SuiteStats;

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
}
