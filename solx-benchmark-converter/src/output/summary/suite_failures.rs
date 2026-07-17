//!
//! One regressed suite's new failures by kind.
//!

///
/// One regressed suite's new failures by kind.
///
#[derive(Debug, PartialEq)]
pub struct SuiteFailures {
    pub label: String,
    pub new_build: usize,
    pub new_test: usize,
}
