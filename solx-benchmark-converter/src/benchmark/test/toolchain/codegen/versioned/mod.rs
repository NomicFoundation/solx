//!
//! Groups test runs by the language version associated with them.
//!

pub mod executable;

use std::collections::BTreeMap;

use self::executable::Executable;
use serde::Deserialize;
use serde::Serialize;

///
/// Encoded compiler mode. In future, it can be replaced with a structured type
/// shared between crates `benchmark_converter` and `compiler_tester`.
///
pub type Mode = String;

///
/// Groups test runs by the language version associated with them.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionedGroup {
    #[serde(flatten)]
    /// Compiled executables associated with test runs.
    pub executables: BTreeMap<Option<Mode>, Executable>,
}
