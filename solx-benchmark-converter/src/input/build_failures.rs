//!
//! Foundry size build failures report.
//!

///
/// Foundry size build failures report.
///
#[derive(Debug, serde::Deserialize)]
pub struct BuildFailuresReport(pub usize);
