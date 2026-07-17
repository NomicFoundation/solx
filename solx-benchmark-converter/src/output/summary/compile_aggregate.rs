//!
//! Compile-time totals for one pipeline.
//!

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
