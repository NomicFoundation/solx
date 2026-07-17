//!
//! Markdown summary of an integration-test benchmark comparison.
//!
//! Renders the one-comment PR summary the integration workflow posts: the
//! correctness verdict (bytecode size everywhere + solx-tester gas), new
//! failures vs main, and a threshold-gated compile-time tripwire. The verdict
//! is computed here — the single source of truth shared by every suite —
//! instead of parsing the XLSX back offline.
//!
//! `SuiteStats` reduces each suite's benchmark to numbers, the verdict types
//! (`OutputVerdict`, `FailureVerdict`, `CompileView`) turn those numbers into
//! typed decisions, and `SummaryTemplate` renders the decisions as markdown.
//!
//! Golden tests pin full rendered comments under `src/tests/fixtures/`;
//! after an intended output change, regenerate them with
//! `UPDATE_SUMMARY_FIXTURES=1 cargo test -p solx-benchmark-converter`.
//!

pub mod compile_aggregate;
pub mod diff_counter;
pub mod failure_regressions;
pub mod paired_bytes;
pub mod suite_failures;
pub mod suite_row;
pub mod suite_stats;
pub mod summary_template;
pub mod top_movers;

use crate::summary_suite::SummarySuite;

use self::suite_stats::SuiteStats;
use self::summary_template::SummaryTemplate;

///
/// The suites a single PR summary comment is rendered from.
///
pub struct Summary {
    suites: Vec<SummarySuite>,
}

impl Summary {
    /// Collects the suites the workflow fed in.
    pub fn new(suites: Vec<SummarySuite>) -> Self {
        Self { suites }
    }

    /// Renders the full PR summary comment.
    pub fn render(&self) -> String {
        let stats: Vec<SuiteStats> = self.suites.iter().map(SuiteStats::from_suite).collect();
        SummaryTemplate::rendered(&stats)
    }

    /// Whether no suite was fed in — nothing to summarize.
    pub fn is_empty(&self) -> bool {
        self.suites.is_empty()
    }
}
