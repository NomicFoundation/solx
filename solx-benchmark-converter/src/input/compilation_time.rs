//!
//! Foundry size compilation time report.
//!

///
/// Foundry size compilation time report.
///
#[derive(Debug, serde::Deserialize)]
pub struct CompilationTimeReport(pub u64);
