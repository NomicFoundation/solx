//!
//! One suite fed into the summary: solx-tester, Foundry, or Hardhat.
//!

use crate::benchmark::Benchmark;
use crate::suite_kind::SuiteKind;
use crate::suite_outcome::SuiteOutcome;

///
/// One suite fed into the summary: solx-tester, Foundry, or Hardhat.
///
pub struct SummarySuite {
    /// Which suite this is. Its label, report file, gas gating, and toolchain
    /// matrix all follow from the kind rather than being restated here, so a
    /// suite cannot contradict its own identity.
    pub kind: SuiteKind,
    /// The merged benchmark holding every toolchain's runs. `None` when the
    /// suite was expected but produced no report because it errored before
    /// writing. Rendered as an explicit failed row rather than silently dropped.
    pub benchmark: Option<Benchmark>,
    /// Artifact download URL for the XLSX report, if uploaded.
    pub report_url: Option<String>,
    /// How the suite's workflow step ended.
    pub outcome: SuiteOutcome,
}
