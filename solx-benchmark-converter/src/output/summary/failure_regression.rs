//!
//! One row whose PR run failed more than its main counterpart.
//!

///
/// One row whose PR run failed more than its main counterpart.
///
pub(crate) struct FailureRegression {
    pub(crate) label: String,
    pub(crate) mode: String,
    pub(crate) kind: &'static str,
    pub(crate) main: usize,
    pub(crate) pr: usize,
}
