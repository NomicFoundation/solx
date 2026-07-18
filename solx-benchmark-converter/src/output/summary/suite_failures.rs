//!
//! One regressed suite's new failures by kind.
//!

///
/// One regressed suite's new failures by kind.
///
#[derive(Debug, PartialEq)]
pub struct SuiteFailures {
    /// The suite label.
    pub label: String,
    /// New build failures in excess of `main`.
    pub new_build: usize,
    /// New test failures in excess of `main`.
    pub new_test: usize,
}

impl SuiteFailures {
    /// The "+N build, +N test" list, one wording shared by the verdict line
    /// and the failures table cell.
    pub fn kinds(build: usize, test: usize) -> String {
        let mut kinds = Vec::new();
        if build > 0 {
            kinds.push(format!("+{} build", crate::utils::commas(build as u64)));
        }
        if test > 0 {
            kinds.push(format!("+{} test", crate::utils::commas(test as u64)));
        }
        kinds.join(", ")
    }
}
