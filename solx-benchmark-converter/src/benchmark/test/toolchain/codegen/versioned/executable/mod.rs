//!
//! Executable is the compiled artifact corresponding to the test.
//! Executables differ by compilation flags.
//!

pub mod metadata;
pub mod run;

use self::metadata::Metadata;
use self::run::Run;
use serde::Deserialize;
use serde::Serialize;

///
/// Executable is the compiled artifact corresponding to the test.
/// Executables differ by compilation flags.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Executable {
    #[serde(default, skip)]
    /// Metadata associated with the compiled executable.
    pub metadata: Metadata,
    #[serde(flatten)]
    /// Measurements.
    pub run: Run,
}
