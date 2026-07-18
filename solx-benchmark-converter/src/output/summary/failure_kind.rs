//!
//! Whether a failure regression is a build failure or a test failure.
//!

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

///
/// Whether a failure regression is a build failure or a test failure.
///
pub enum FailureKind {
    /// A compilation failure.
    Build,
    /// A test-execution failure.
    Test,
}

impl Display for FailureKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::Build => write!(formatter, "build"),
            Self::Test => write!(formatter, "test"),
        }
    }
}
