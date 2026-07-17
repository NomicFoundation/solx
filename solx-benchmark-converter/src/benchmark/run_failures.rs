//!
//! What a project-level run did, and how many failures it produced.
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
    /// never ran, an absence that is not a measured zero.
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
