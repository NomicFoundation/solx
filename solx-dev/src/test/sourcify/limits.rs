//!
//! `solx` Sourcify sweep limits.
//!

use std::num::NonZeroUsize;
use std::time::Duration;

///
/// The compiler invocation limits of a sweep.
///
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The number of concurrent compiler processes, or the available parallelism.
    pub threads: Option<NonZeroUsize>,
    /// The per-compilation timeout.
    pub timeout: Duration,
    /// The resident memory cap of each compiler with its workers in MiB.
    pub memory_limit_mib: Option<u64>,
}
