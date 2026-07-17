//!
//! One row of the results table, a cell per column.
//!

///
/// One row of the results table, a cell per column.
///
pub struct SuiteRow {
    pub suite: String,
    pub failures: String,
    pub size: String,
    pub gas: String,
    pub report: String,
}
