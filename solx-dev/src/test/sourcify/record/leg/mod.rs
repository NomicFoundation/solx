//!
//! `solx` Sourcify sweep compiler leg.
//!

pub mod kind;

use std::collections::BTreeMap;

use self::kind::Kind;

///
/// One compiler invocation on one corpus contract.
///
/// The report re-buckets from the recorded messages and standard error tail, so the bucketing can
/// be refined and re-rendered over old results.
///
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Leg {
    /// How the invocation ended.
    pub kind: Kind,
    /// The exit code, absent when a signal killed the process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<i32>,
    /// The signal that killed the process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,
    /// The first diagnostic with paths, library hashes and numbers normalized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// The tail of the standard error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    /// The first error-severity messages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
    /// The number of error-severity messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_count: Option<usize>,
    /// The contracts with bytecode per source path, recorded when the target got none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emitted: Option<BTreeMap<String, Vec<String>>>,
}
