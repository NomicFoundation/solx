//!
//! One row of the results table, a cell per column.
//!

///
/// One row of the results table, a cell per column.
///
pub struct SuiteRow {
    /// The suite-name cell.
    pub suite: String,
    /// The failures-verdict cell.
    pub failures: String,
    /// The bytecode-size cell.
    pub size: String,
    /// The gas cell.
    pub gas: String,
    /// The report-link cell.
    pub report: String,
}
