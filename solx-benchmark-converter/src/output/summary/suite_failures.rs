//!
//! One regressed suite's new failures by kind.
//!

///
/// One regressed suite's new failures by kind.
///
#[derive(Debug, PartialEq)]
pub(crate) struct SuiteFailures {
    pub(crate) label: String,
    pub(crate) new_build: usize,
    pub(crate) new_test: usize,
}
