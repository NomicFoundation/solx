//!
//! `solx` Sourcify sweep result record.
//!

use std::collections::BTreeMap;

///
/// One corpus contract's sweep result, written as a JSON line.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// The corpus file stem: `<chain>_<address>`.
    #[serde(rename = "id")]
    pub identifier: String,
    /// The chain the contract is deployed on.
    #[serde(default)]
    pub chain_id: serde_json::Value,
    /// The source path whose contracts must get bytecode.
    pub target: String,
    /// The candidate compiler leg.
    pub slang: Leg,
    /// The reference compiler leg, run only when the candidate fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Leg>,
    /// The overall outcome.
    pub outcome: Outcome,
}

///
/// The overall outcome of one corpus contract.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// The candidate compiles.
    Ok,
    /// The candidate fails and the reference compiles.
    SlangFail,
    /// The candidate and the reference both fail.
    BothFail,
    /// The candidate exceeded the per-contract timeout.
    Timeout,
}

impl Outcome {
    /// All outcomes in report order.
    pub const ALL: [Self; 4] = [Self::Ok, Self::SlangFail, Self::BothFail, Self::Timeout];

    ///
    /// The serialized name.
    ///
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::SlangFail => "slang-fail",
            Self::BothFail => "both-fail",
            Self::Timeout => "timeout",
        }
    }

    ///
    /// The meaning shown in the report's outcome table.
    ///
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Ok => "compiles: no errors and bytecode for the target source",
            Self::SlangFail => "candidate fails, baseline compiles (or no baseline given)",
            Self::BothFail => {
                "candidate and baseline both fail: corpus artifact or documented compiler limitation"
            }
            Self::Timeout => "candidate exceeded the per-contract timeout",
        }
    }
}

///
/// One compiler invocation on one corpus contract.
///
/// Only raw material is recorded; the report buckets it, so the bucketing can be refined and
/// re-rendered over old results.
///
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Leg {
    /// How the invocation ended.
    pub kind: Kind,
    /// The exit code, or the negated signal number if the process was killed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit: Option<i32>,
    /// The first diagnostic with paths and numbers normalized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// The tail of the standard error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    /// The first error-severity messages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
    /// The number of error-severity messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_count: Option<usize>,
    /// The contracts with bytecode per source path, recorded when the target got none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emitted: Option<BTreeMap<String, Vec<String>>>,
}

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
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// No error-severity diagnostics and bytecode for the target source.
    #[default]
    Ok,
    /// Error-severity diagnostics, or an exit without standard JSON output.
    Error,
    /// A clean exit without bytecode for the target source.
    NoBytecode,
    /// A Rust panic.
    Panic,
    /// Killed by a signal, e.g. an LLVM fatal error or the memory limit.
    Abort,
    /// The per-contract timeout was exceeded.
    Timeout,
}

impl Kind {
    ///
    /// The serialized name.
    ///
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::NoBytecode => "no-bytecode",
            Self::Panic => "panic",
            Self::Abort => "abort",
            Self::Timeout => "timeout",
        }
    }
}
