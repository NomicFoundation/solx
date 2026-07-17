//!
//! Whether the PR preserved compiler output, judged over every suite's size
//! comparisons plus the gas comparisons of gating suites.
//!

use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::gas_change::GasChange;
use crate::output::summary::size_change::SizeChange;
use crate::output::summary::suite_stats::SuiteStats;
use crate::utils::agreeing;
use crate::utils::commas;
use crate::utils::count_noun;
use crate::utils::signed_commas;

///
/// Whether the PR preserved compiler output, judged over every suite's size
/// comparisons plus the gas comparisons of gating suites. Non-gated gas
/// (fuzz-noisy Foundry/Hardhat runs) never influences this verdict.
///
#[derive(Debug, PartialEq)]
pub enum OutputVerdict {
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
    pub fn from_stats(stats: &[SuiteStats]) -> Self {
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

    /// The output-invariance verdict line.
    pub fn line(self) -> String {
        match self {
            Self::NoData => {
                "⚪ **No output data** — no size or gated-gas comparison had a `main` counterpart \
                 to compare against."
                    .to_owned()
            }
            Self::Preserving {
                size_cells,
                gated_gas_cells,
                gas_label,
            } => {
                let mut clauses = Vec::new();
                if size_cells > 0 {
                    clauses.push(format!(
                        "bytecode size identical ({})",
                        count_noun(size_cells, "comparison")
                    ));
                }
                if gated_gas_cells > 0 {
                    clauses.push(format!(
                        "{gas_label} gas identical ({})",
                        commas(gated_gas_cells)
                    ));
                }
                format!("✅ **Output-preserving** — {}.", clauses.join(", "))
            }
            Self::Changed { size, gas } => {
                let mut parts = Vec::new();
                if let Some(size) = size {
                    parts.push(format!(
                        "{} of {} {} ({} B total)",
                        commas(size.diffs),
                        count_noun(size.cells, "size comparison"),
                        agreeing(size.diffs, "differs", "differ"),
                        signed_commas(size.delta_bytes)
                    ));
                }
                if let Some(gas) = gas {
                    parts.push(format!(
                        "{} of {} {}",
                        commas(gas.diffs),
                        count_noun(gas.cells, format!("{} gas comparison", gas.label).as_str()),
                        agreeing(gas.diffs, "differs", "differ")
                    ));
                }
                format!(
                    "⚠️ **Output changed** — {}. If this PR is meant to be output-preserving, \
                     investigate before merging.",
                    parts.join("; ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::output::summary::output_verdict::*;
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
