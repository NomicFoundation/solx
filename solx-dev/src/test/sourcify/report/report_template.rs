//!
//! `solx` Sourcify sweep report template.
//!
//! The report's shape lives in `templates/sourcify-report.md`; everything the template
//! interpolates is a string precomputed here, so the template tests presence and never formats.
//!

use askama::Template;

use crate::test::sourcify::record::Record;
use crate::test::sourcify::record::outcome::Outcome;

use super::census::Census;
use super::listing::Listing;

/// The number of timed-out contracts listed.
const TIMEOUT_EXAMPLES: usize = 20;

///
/// The full report.
///
#[derive(Template)]
#[template(path = "sourcify-report.md", escape = "none")]
pub struct ReportTemplate {
    /// The report heading.
    pub title: String,
    /// One line per compiler, with its name and version.
    pub compiler_lines: Vec<String>,
    /// The number of contracts swept.
    pub contracts: String,
    /// The outcome table rows: outcome, count, share, meaning.
    pub outcome_rows: Vec<Vec<String>>,
    /// The failure censuses, candidate-only first.
    pub censuses: Vec<Census>,
    /// The timed-out contracts.
    pub timeouts: Option<Listing>,
}

impl ReportTemplate {
    ///
    /// Everything the template interpolates, precomputed from the sorted records.
    ///
    pub fn new(
        title: String,
        meta: &serde_json::Map<String, serde_json::Value>,
        records: &[Record],
    ) -> Self {
        let total = records.len();
        let compiler_lines = ["candidate", "reference"]
            .into_iter()
            .filter_map(|name| {
                let version = meta.get(name).and_then(serde_json::Value::as_str)?;
                Some(format!("{name}: `{version}`"))
            })
            .collect();
        let outcome_rows = Outcome::ALL
            .into_iter()
            .filter_map(|outcome| {
                let count = records
                    .iter()
                    .filter(|record| record.outcome == outcome)
                    .count();
                (count > 0).then(|| {
                    vec![
                        outcome.to_string(),
                        count.to_string(),
                        format!("{:.2}%", count as f64 / total as f64 * 100.0),
                        outcome.meaning().to_owned(),
                    ]
                })
            })
            .collect();

        let candidate_only: Vec<_> = records
            .iter()
            .filter(|record| record.outcome == Outcome::CandidateFail)
            .map(|record| (record.identifier.as_str(), &record.candidate))
            .collect();
        // For shared failures the reference's diagnostic is the one that explains the contract.
        let shared: Vec<_> = records
            .iter()
            .filter(|record| record.outcome == Outcome::BothFail)
            .map(|record| {
                (
                    record.identifier.as_str(),
                    record
                        .reference
                        .as_ref()
                        .expect("Always exists for both-fail"),
                )
            })
            .collect();
        let censuses = [
            Census::new("Candidate-only failures", candidate_only.as_slice()),
            Census::new(
                "Failures shared with the reference (reference diagnostics)",
                shared.as_slice(),
            ),
        ]
        .into_iter()
        .flatten()
        .collect();

        let timeouts: Vec<&str> = records
            .iter()
            .filter(|record| record.outcome == Outcome::Timeout)
            .map(|record| record.identifier.as_str())
            .collect();
        let timeouts =
            (!timeouts.is_empty()).then(|| Listing::new(timeouts.as_slice(), TIMEOUT_EXAMPLES));

        Self {
            title,
            compiler_lines,
            contracts: total.to_string(),
            outcome_rows,
            censuses,
            timeouts,
        }
    }
}
