//!
//! Whether the PR preserved compiler output, judged over every suite's size
//! comparisons plus the gas comparisons of gating suites.
//!

pub mod gas_change;
pub mod size_change;

use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::suite_stats::SuiteStats;
use crate::utils;

use self::gas_change::GasChange;
use self::size_change::SizeChange;

///
/// Whether the PR preserved compiler output, judged over every suite's size
/// comparisons plus the gas comparisons of gating suites. Non-gated gas, from
/// fuzz-noisy Foundry/Hardhat runs, never influences this verdict.
///
#[derive(Debug, PartialEq)]
pub enum OutputVerdict {
    /// No size or gated-gas comparison paired a PR value with a `main` one,
    /// whether nothing was collected or everything collected was one-sided:
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
                        utils::count_noun(size_cells, "comparison")
                    ));
                }
                if gated_gas_cells > 0 {
                    clauses.push(format!(
                        "{gas_label} gas identical ({})",
                        utils::commas(gated_gas_cells)
                    ));
                }
                format!("✅ **Output-preserving** — {}.", clauses.join(", "))
            }
            Self::Changed { size, gas } => {
                let mut parts = Vec::new();
                if let Some(size) = size {
                    parts.push(format!(
                        "{} of {} {} ({} B total)",
                        utils::commas(size.diffs),
                        utils::count_noun(size.cells, "size comparison"),
                        utils::agreeing(size.diffs, "differs", "differ"),
                        utils::signed_commas(size.delta_bytes)
                    ));
                }
                if let Some(gas) = gas {
                    parts.push(format!(
                        "{} of {} {}",
                        utils::commas(gas.diffs),
                        utils::count_noun(
                            gas.cells,
                            format!("{} gas comparison", gas.label).as_str()
                        ),
                        utils::agreeing(gas.diffs, "differs", "differ")
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
