//!
//! The suites the workflow feeds in.
//!

use std::path::PathBuf;

use crate::benchmark::Benchmark;
use crate::suite_outcome::SuiteOutcome;
use crate::summary_suite::SummarySuite;
use crate::toolchain_matrix::ToolchainMatrix;

///
/// The suites the workflow feeds in — the one place owning each suite's
/// label, file names, gas gating, and toolchain matrix, so the binary, the
/// harness writers, and the tests cannot drift apart (a gas-gated Project
/// suite is unrepresentable).
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiteKind {
    /// solx-tester: deterministic REVM gas, tester naming matrix.
    Tester,
    /// Foundry projects: fuzz-noisy gas, project naming matrix.
    Foundry,
    /// Hardhat projects: fuzz-noisy gas, project naming matrix.
    Hardhat,
}

impl SuiteKind {
    /// The human-readable suite name shown in the table.
    pub fn label(self) -> &'static str {
        match self {
            Self::Tester => "solx-tester",
            Self::Foundry => "Foundry",
            Self::Hardhat => "Hardhat",
        }
    }

    /// The XLSX report file name inside the uploaded artifact.
    pub fn report_file(self) -> &'static str {
        match self {
            Self::Tester => "solx-tester-report.xlsx",
            Self::Foundry => "foundry-report.xlsx",
            Self::Hardhat => "hardhat-report.xlsx",
        }
    }

    /// The merged benchmark JSON file name the summary step reads; the
    /// workflow spells these paths in `integration-tests.yaml`.
    pub fn benchmark_file(self) -> &'static str {
        match self {
            Self::Tester => "solx-tester-benchmark.json",
            Self::Foundry => "foundry-benchmark.json",
            Self::Hardhat => "hardhat-benchmark.json",
        }
    }

    /// Whether the suite's gas is deterministic and gates correctness.
    pub fn gas_is_gate(self) -> bool {
        matches!(self, Self::Tester)
    }

    /// Which toolchain naming matrix the suite's mode strings follow.
    pub fn matrix(self) -> ToolchainMatrix {
        match self {
            Self::Tester => ToolchainMatrix::Tester,
            Self::Foundry | Self::Hardhat => ToolchainMatrix::Project,
        }
    }

    ///
    /// Loads this suite for the summary, or `None` when it was not part of the
    /// invocation (no benchmark and not skipped). A skipped outcome renders a
    /// "did not run" row; a benchmark path that will not parse renders the
    /// suite as errored rather than aborting the summary for the healthy ones.
    /// A skipped upload step passes its URL through as an empty string, which
    /// is treated as no URL.
    ///
    pub fn load(
        self,
        path: Option<PathBuf>,
        report_url: Option<String>,
        outcome: SuiteOutcome,
    ) -> Option<SummarySuite> {
        let benchmark = match (outcome, path) {
            (SuiteOutcome::Skipped, _) => None,
            (_, None) => return None,
            (_, Some(path)) => match Benchmark::try_from(path.as_path()) {
                Ok(benchmark) => Some(benchmark),
                Err(error) => {
                    eprintln!(
                        "Warning: {} benchmark is unusable ({error}); rendering the suite as errored.",
                        self.label()
                    );
                    None
                }
            },
        };
        Some(SummarySuite {
            kind: self,
            benchmark,
            report_url: report_url.filter(|url| !url.is_empty()),
            outcome,
        })
    }
}
