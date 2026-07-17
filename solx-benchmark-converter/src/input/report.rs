//!
//! The various benchmark report formats from tooling.
//!

use crate::benchmark::Benchmark;
use crate::input::build_failures::BuildFailuresReport;
use crate::input::compilation_time::CompilationTimeReport;
use crate::input::foundry_gas::FoundryGasReport;
use crate::input::foundry_size::FoundrySizeReport;
use crate::input::test_failures::TestFailuresReport;
use crate::input::testing_time::TestingTimeReport;

///
/// Enum representing various benchmark formats from tooling.
///
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Report {
    /// Benchmark converter's native benchmark report format.
    Native(Benchmark),
    /// Foundry gas report.
    FoundryGas(FoundryGasReport),
    /// Foundry size report.
    FoundrySize(FoundrySizeReport),
    /// Compilation time report.
    CompilationTime(CompilationTimeReport),
    /// Testing time report.
    TestingTime(TestingTimeReport),
    /// Build failures report.
    BuildFailures(BuildFailuresReport),
    /// Test failures report.
    TestFailures(TestFailuresReport),
}

impl From<Benchmark> for Report {
    fn from(report: Benchmark) -> Self {
        Self::Native(report)
    }
}

impl From<FoundryGasReport> for Report {
    fn from(report: FoundryGasReport) -> Self {
        Self::FoundryGas(report)
    }
}

impl From<FoundrySizeReport> for Report {
    fn from(report: FoundrySizeReport) -> Self {
        Self::FoundrySize(report)
    }
}

impl From<CompilationTimeReport> for Report {
    fn from(report: CompilationTimeReport) -> Self {
        Self::CompilationTime(report)
    }
}

impl From<TestingTimeReport> for Report {
    fn from(report: TestingTimeReport) -> Self {
        Self::TestingTime(report)
    }
}

impl From<BuildFailuresReport> for Report {
    fn from(report: BuildFailuresReport) -> Self {
        Self::BuildFailures(report)
    }
}

impl From<TestFailuresReport> for Report {
    fn from(report: TestFailuresReport) -> Self {
        Self::TestFailures(report)
    }
}
