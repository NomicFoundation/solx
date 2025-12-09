//!
//! Foundry size test failures report.
//!

///
/// Foundry size test failures report.
///
#[derive(Debug, serde::Deserialize)]
pub struct TestFailuresReport(pub usize);
