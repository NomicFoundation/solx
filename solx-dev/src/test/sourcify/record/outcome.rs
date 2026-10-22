//!
//! `solx` Sourcify sweep contract outcome.
//!

///
/// The overall outcome of one corpus contract.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    /// The meaning shown in the report's outcome table.
    ///
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Ok => "compiles: successful exit, no errors and bytecode for the target source",
            Self::SlangFail => "candidate fails, baseline compiles",
            Self::BothFail => "candidate and baseline both fail",
            Self::Timeout => "candidate exceeded the per-contract timeout",
        }
    }
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "ok"),
            Self::SlangFail => write!(f, "slang-fail"),
            Self::BothFail => write!(f, "both-fail"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}
