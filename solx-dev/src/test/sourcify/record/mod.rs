//!
//! `solx` Sourcify sweep result record.
//!

pub mod leg;
pub mod outcome;

use self::leg::Leg;
use self::outcome::Outcome;

///
/// One corpus contract's sweep result, written as a JSON line.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Record {
    /// The corpus file stem: `<chain>_<address>`.
    #[serde(rename = "id")]
    pub identifier: String,
    /// The source path whose contracts must get bytecode.
    pub target: String,
    /// The candidate compiler leg.
    pub slang: Leg,
    /// The reference compiler leg, run only when the candidate fails without timing out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<Leg>,
    /// The overall outcome.
    pub outcome: Outcome,
}
