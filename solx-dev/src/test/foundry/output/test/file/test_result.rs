//!
//! Foundry test output report result.
//!

///
/// Foundry test output report result.
///
#[derive(Debug, serde::Deserialize)]
pub struct TestResult {
    /// Test status: usually "Success" or "Failure".
    pub status: String,
    /// Test failure reason, if any.
    pub reason: Option<String>,
}
