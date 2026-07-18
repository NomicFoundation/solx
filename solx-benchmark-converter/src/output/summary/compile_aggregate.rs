//!
//! Compile-time totals for one pipeline.
//!

///
/// Compile-time totals for one pipeline.
///
#[derive(Default)]
pub struct CompileAggregate {
    /// The PR's summed compile time, in milliseconds.
    pub pr_total_ms: u64,
    /// `main`'s summed compile time, in milliseconds.
    pub main_total_ms: u64,
    /// Per-project percentage change, PR vs main.
    pub per_project: Vec<(String, f64)>,
}

impl CompileAggregate {
    /// A suite-aggregate compile-time move at least this large is highlighted.
    pub const SUITE_THRESHOLD_PERCENT: f64 = 5.0;

    /// One aggregate/median compile cell, and whether it crossed the suite
    /// threshold. Both directions defeat "within noise", but only a slowdown
    /// gets the siren. A large improvement is signal, not an alarm.
    pub fn cell(&self, percentage: f64) -> (String, bool) {
        let (aggregate, flagged) = if percentage >= Self::SUITE_THRESHOLD_PERCENT {
            (
                format!("⚠️ **{}**", crate::utils::percent(percentage)),
                true,
            )
        } else if percentage <= -Self::SUITE_THRESHOLD_PERCENT {
            (format!("**{}**", crate::utils::percent(percentage)), true)
        } else {
            (crate::utils::percent(percentage), false)
        };
        let project_percentages: Vec<f64> = self
            .per_project
            .iter()
            .map(|(_, percentage)| *percentage)
            .collect();
        match crate::utils::median(project_percentages.as_slice()) {
            Some(median) => (
                format!("{aggregate} / {}", crate::utils::percent(median)),
                flagged,
            ),
            None => (aggregate, flagged),
        }
    }
}
