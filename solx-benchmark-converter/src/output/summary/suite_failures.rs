//!
//! One regressed suite's new failures by kind.
//!

use crate::utils::commas;

///
/// One regressed suite's new failures by kind.
///
#[derive(Debug, PartialEq)]
pub struct SuiteFailures {
    pub label: String,
    pub new_build: usize,
    pub new_test: usize,
}

impl SuiteFailures {
    /// The "+N build, +N test" list — one wording shared by the verdict line
    /// and the failures table cell.
    pub fn kinds(build: usize, test: usize) -> String {
        let mut kinds = Vec::new();
        if build > 0 {
            kinds.push(format!("+{} build", commas(build as u64)));
        }
        if test > 0 {
            kinds.push(format!("+{} test", commas(test as u64)));
        }
        kinds.join(", ")
    }
}
