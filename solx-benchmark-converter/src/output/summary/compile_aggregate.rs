//!
//! Compile-time totals for one pipeline.
//!

use crate::utils::median;
use crate::utils::percent;

///
/// Compile-time totals for one pipeline.
///
#[derive(Default)]
pub struct CompileAggregate {
    pub pr_total_ms: u64,
    pub main_total_ms: u64,
    /// Per-project percentage change, PR vs main.
    pub per_project: Vec<(String, f64)>,
}

impl CompileAggregate {
    /// A suite-aggregate compile-time move at least this large is highlighted.
    pub const SUITE_THRESHOLD_PERCENT: f64 = 5.0;

    /// One aggregate/median compile cell, and whether it crossed the suite
    /// threshold. Both directions defeat "within noise", but only a slowdown
    /// gets the siren — a large improvement is signal, not an alarm.
    pub fn cell(&self, pct: f64) -> (String, bool) {
        let (aggregate, flagged) = if pct >= Self::SUITE_THRESHOLD_PERCENT {
            (format!("⚠️ **{}**", percent(pct)), true)
        } else if pct <= -Self::SUITE_THRESHOLD_PERCENT {
            (format!("**{}**", percent(pct)), true)
        } else {
            (percent(pct), false)
        };
        let project_pcts: Vec<f64> = self.per_project.iter().map(|(_, pct)| *pct).collect();
        match median(project_pcts.as_slice()) {
            Some(med) => (format!("{aggregate} / {}", percent(med)), flagged),
            None => (aggregate, flagged),
        }
    }
}
