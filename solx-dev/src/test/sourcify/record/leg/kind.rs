//!
//! `solx` Sourcify sweep compiler invocation kind.
//!

///
/// How one compiler invocation ended.
///
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// A successful exit, no error-severity diagnostics and bytecode for the target source.
    #[default]
    Ok,
    /// Error-severity diagnostics, a failing exit, or an exit without standard JSON output.
    Error,
    /// A clean exit without bytecode for the target source.
    NoBytecode,
    /// A Rust panic.
    Panic,
    /// Killed by a signal, e.g. an LLVM fatal error, or by the memory limit.
    Abort,
    /// The per-contract timeout was exceeded.
    Timeout,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::Error => write!(f, "error"),
            Self::NoBytecode => write!(f, "no-bytecode"),
            Self::Panic => write!(f, "panic"),
            Self::Abort => write!(f, "abort"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}
