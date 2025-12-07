//!
//! Foundry test output report file test result.
//!

pub mod test_result;

use std::collections::BTreeMap;

use self::test_result::TestResult;

///
/// Foundry test output report file test result.
///
#[derive(Debug, serde::Deserialize)]
pub struct File {
    /// Test results mapping.
    pub test_results: BTreeMap<String, TestResult>,
}
