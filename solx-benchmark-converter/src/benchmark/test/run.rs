//!
//! A run of a test with fixed compiler options (mode).
//!

use serde::Deserialize;
use serde::Serialize;

///
/// What a project-level run did, and how many failures it produced. A
/// toolchain whose build failed never reaches its tests: both runners push
/// their build failures and skip the test report, so a build failure and a
/// test-failure count cannot coexist on one run.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunFailures {
    /// The build failed with this many errors; the tests never ran.
    Build(usize),
    /// The build succeeded; the tests ran and this many failed.
    Test(usize),
}

impl RunFailures {
    ///
    /// Build failures count. A run that reached its tests built cleanly.
    ///
    pub fn build_failures(self) -> usize {
        match self {
            Self::Build(count) => count,
            Self::Test(_) => 0,
        }
    }

    ///
    /// Test failures count, or `None` where the build failed and the tests
    /// never ran — an absence that is not a measured zero.
    ///
    pub fn test_failures(self) -> Option<usize> {
        match self {
            Self::Build(_) => None,
            Self::Test(count) => Some(count),
        }
    }

    ///
    /// Every failure the run observed, whichever stage produced it.
    ///
    pub fn count(self) -> usize {
        match self {
            Self::Build(count) | Self::Test(count) => count,
        }
    }
}

///
/// Run of a test with specific compiler options.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Run {
    /// Contract deploy code size.
    #[serde(default)]
    pub size: Vec<u64>,
    /// Contract runtime code size.
    #[serde(default)]
    pub runtime_size: Vec<u64>,
    /// Amount of EVM gas.
    #[serde(default)]
    pub gas: Vec<u64>,
    /// Compilation time in milliseconds.
    #[serde(default)]
    pub compilation_time: Vec<u64>,
    /// Testing time in milliseconds.
    #[serde(default)]
    pub testing_time: Vec<u64>,
    /// What the project-level run did. `None` where the run carries no
    /// failure report at all: contract-level runs and the tester's native
    /// reports never produce one.
    #[serde(default)]
    pub failures: Option<RunFailures>,
}

impl Run {
    ///
    /// Extends the run with another run, averaging the values.
    ///
    pub fn extend(&mut self, other: &Self) -> anyhow::Result<()> {
        self.size.extend_from_slice(other.size.as_slice());
        self.runtime_size
            .extend_from_slice(other.runtime_size.as_slice());
        self.gas
            .extend(other.gas.iter().filter(|value| value < &&(u32::MAX as u64)));
        self.compilation_time
            .extend_from_slice(other.compilation_time.as_slice());
        self.testing_time
            .extend_from_slice(other.testing_time.as_slice());
        self.failures = match (self.failures, other.failures) {
            (None, failures) | (failures, None) => failures,
            (Some(RunFailures::Build(left)), Some(RunFailures::Build(right))) => {
                Some(RunFailures::Build(left + right))
            }
            (Some(RunFailures::Test(left)), Some(RunFailures::Test(right))) => {
                Some(RunFailures::Test(left + right))
            }
            (Some(left), Some(right)) => anyhow::bail!(
                "Run merges a build failure with a test result: {left:?} and {right:?}"
            ),
        };
        Ok(())
    }

    ///
    /// Average contract size.
    ///
    pub fn average_size(&self) -> u64 {
        if self.size.is_empty() {
            return 0;
        }

        self.size.iter().sum::<u64>() / (self.size.len() as u64)
    }

    ///
    /// Average runtime code size.
    ///
    pub fn average_runtime_size(&self) -> u64 {
        if self.runtime_size.is_empty() {
            return 0;
        }

        self.runtime_size.iter().sum::<u64>() / (self.runtime_size.len() as u64)
    }

    ///
    /// Average amount of EVM gas.
    ///
    pub fn average_gas(&self) -> u64 {
        if self.gas.is_empty() {
            return 0;
        }

        self.gas.iter().sum::<u64>() / (self.gas.len() as u64)
    }

    ///
    /// Average compilation time in milliseconds.
    ///
    pub fn average_compilation_time(&self) -> u64 {
        if self.compilation_time.is_empty() {
            return 0;
        }

        self.compilation_time.iter().sum::<u64>() / (self.compilation_time.len() as u64)
    }

    ///
    /// Average testing time in milliseconds.
    ///
    pub fn average_testing_time(&self) -> u64 {
        if self.testing_time.is_empty() {
            return 0;
        }

        self.testing_time.iter().sum::<u64>() / (self.testing_time.len() as u64)
    }

    ///
    /// Build failures count, or `None` where the run reported no failures at
    /// all — nothing was measured, which is not a clean build.
    ///
    pub fn build_failures_count(&self) -> Option<usize> {
        self.failures.map(RunFailures::build_failures)
    }

    ///
    /// Test failures count, or `None` where the tests never ran.
    ///
    pub fn test_failures_count(&self) -> Option<usize> {
        self.failures.and_then(RunFailures::test_failures)
    }

    ///
    /// Every failure the run observed. A run that reported none contributes
    /// nothing.
    ///
    pub fn failures_count(&self) -> usize {
        self.failures.map(RunFailures::count).unwrap_or_default()
    }
}
