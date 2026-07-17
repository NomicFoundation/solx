//!
//! Compile-time totals for one pipeline.
//!

///
/// Compile-time totals for one pipeline.
///
#[derive(Default)]
pub(crate) struct CompileAggregate {
    pub(crate) pr_total_ms: u64,
    pub(crate) main_total_ms: u64,
    /// Per-project percentage change, PR vs main.
    pub(crate) per_project: Vec<(String, f64)>,
}
