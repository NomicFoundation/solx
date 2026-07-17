//!
//! One row whose PR run failed more than its main counterpart.
//!

///
/// One row whose PR run failed more than its main counterpart.
///
pub struct FailureRegression {
    pub label: String,
    pub mode: String,
    pub kind: &'static str,
    pub main: usize,
    pub pr: usize,
}
