//!
//! The full summary comment: every string the markdown template interpolates,
//! precomputed from the per-suite statistics.
//!
//! The comment's shape lives in `templates/summary.md`: section order, headers,
//! table pipes, bullet and blank-line discipline; everything the template
//! interpolates is a string precomputed here. The boundary rule: the template
//! may test presence (`if let`, `is_empty`), never magnitude; anything that
//! formats a value is Rust.
//!

pub mod compile_view;
pub mod failure_verdict;
pub mod health_issue;
pub mod listing_section;
pub mod output_verdict;
pub mod truncated;

use std::collections::BTreeSet;

use askama::Template;

use crate::output::summary::suite_row::SuiteRow;
use crate::output::summary::suite_stats::SuiteStats;
use crate::pipeline::Pipeline;
use crate::role::Role;

use self::compile_view::CompileView;
use self::failure_verdict::FailureVerdict;
use self::health_issue::HealthIssue;
use self::listing_section::ListingSection;
use self::output_verdict::OutputVerdict;
use self::truncated::Truncated;

///
/// The full summary comment.
///
#[derive(Template)]
#[template(path = "summary.md", escape = "none")]
pub struct SummaryTemplate {
    /// Whether the run carries released-solx and solc baselines.
    pub full_matrix: bool,
    /// The bytecode-and-gas invariance headline.
    pub output_line: String,
    /// The failure-regression headline.
    pub failures_line: String,
    /// The harness-health lines, one per degraded suite.
    pub health_lines: Vec<String>,
    /// The no-baseline and missing-on-PR warning lines.
    pub warn_lines: Vec<String>,
    /// The results-table rows, one per suite.
    pub suite_rows: Vec<SuiteRow>,
    /// The inline bullets behind a "New failures" verdict.
    pub new_failure_bullets: Vec<String>,
    /// The inline "largest changes" listings behind an "Output changed" verdict.
    pub mover_sections: Vec<ListingSection>,
    /// The compile-time table, when any suite collected compile times.
    pub compile: Option<CompileView>,
    /// The PR-vs-baseline bytecode-size rows.
    pub baseline_rows: Vec<Vec<String>>,
}

impl SummaryTemplate {
    /// Renders the full summary comment for the given per-suite statistics.
    pub fn rendered(stats: &[SuiteStats]) -> String {
        Self::from_stats(stats)
            .render()
            .expect("template rendering writes to a String")
    }

    /// Everything the template interpolates, precomputed from the per-suite
    /// statistics.
    fn from_stats(stats: &[SuiteStats]) -> Self {
        let (health_lines, warn_lines) = Self::health_lines(stats);
        Self {
            full_matrix: stats.iter().any(|suite| suite.has_baselines),
            output_line: OutputVerdict::from_stats(stats).line(),
            failures_line: FailureVerdict::from_stats(stats).line(),
            health_lines,
            warn_lines,
            suite_rows: stats.iter().map(SuiteStats::row).collect(),
            new_failure_bullets: Self::new_failure_bullets(stats),
            mover_sections: ListingSection::from_stats(stats),
            compile: CompileView::from_stats(stats),
            baseline_rows: Self::baseline_rows(stats),
        }
    }

    /// The harness-health lines, plus the aggregated no-baseline line that
    /// closes the verdict block.
    fn health_lines(stats: &[SuiteStats]) -> (Vec<String>, Vec<String>) {
        let mut lines = Vec::new();
        let mut unbaselined = Vec::new();
        let mut unbaselined_runs = 0;
        let mut unbaselined_failures = 0;
        let mut main_only = Vec::new();
        let mut main_only_runs = 0;
        for issue in HealthIssue::from_stats(stats) {
            match issue {
                HealthIssue::SuiteErrored { label } => {
                    lines.push(format!(
                        "❌ **Suite errored** — {label} produced no usable report."
                    ));
                }
                HealthIssue::StepFailed { label } => {
                    lines.push(format!(
                        "⚠️ **Suite step failed** — {label} exited nonzero after its report was \
                         written; results may be incomplete."
                    ));
                }
                HealthIssue::EmptySuite { label } => {
                    lines.push(format!(
                        "❌ **Suite empty** — {label}'s report contains no runs."
                    ));
                }
                HealthIssue::UnrecognizedToolchains { label } => {
                    lines.push(format!(
                        "❌ **Harness error** — {label}: benchmark data matched no recognized \
                         toolchain naming."
                    ));
                }
                HealthIssue::UnrecognizedRuns { label, modes } => {
                    lines.push(format!(
                        "❌ **Harness error** — {label}: runs matched no declared toolchain: {}.",
                        Self::truncated_mode_list(&modes)
                    ));
                }
                HealthIssue::UnrecognizedPipelines { label, modes } => {
                    lines.push(format!(
                        "❌ **Harness error** — {label}: no recognized pipeline token in: {}.",
                        Self::truncated_mode_list(&modes)
                    ));
                }
                HealthIssue::Unbaselined {
                    label,
                    runs,
                    failures,
                } => {
                    unbaselined_runs += runs;
                    unbaselined_failures += failures;
                    unbaselined.push(Self::unpaired_runs_part(&label, runs, failures));
                }
                HealthIssue::MainOnly {
                    label,
                    runs,
                    failures,
                } => {
                    main_only_runs += runs;
                    main_only.push(Self::unpaired_runs_part(&label, runs, failures));
                }
            }
        }
        let mut warn_lines = Vec::new();
        if !unbaselined.is_empty() {
            warn_lines.push(format!(
                "⚠️ **No baseline** — {} {} no `main` counterpart; {} {} not compared.",
                unbaselined.join("; "),
                crate::utils::agreeing(unbaselined_runs as u64, "has", "have"),
                crate::utils::agreeing(unbaselined_runs as u64, "its", "their"),
                crate::utils::agreeing(unbaselined_failures as u64, "failure is", "failures are")
            ));
        }
        if !main_only.is_empty() {
            warn_lines.push(format!(
                "⚠️ **Missing on PR** — {} {} only on `main`; the comparison set shrank.",
                main_only.join("; "),
                crate::utils::agreeing(main_only_runs as u64, "exists", "exist")
            ));
        }
        (lines, warn_lines)
    }

    /// One suite's share of an unpaired-runs warning line, the same wording
    /// for the no-baseline and main-only directions.
    fn unpaired_runs_part(label: &str, runs: usize, failures: usize) -> String {
        format!(
            "{label}: {} ({})",
            crate::utils::count_noun(runs as u64, "run"),
            crate::utils::count_noun(failures as u64, "failure")
        )
    }

    /// Backtick-quoted mode strings, capped with a "+N more".
    fn truncated_mode_list(modes: &[String]) -> String {
        let truncated = Truncated::new(modes);
        let shown: Vec<String> = truncated
            .shown
            .iter()
            .map(|mode| format!("`{mode}`"))
            .collect();
        format!("{}{}", shown.join(", "), truncated.more_suffix())
    }

    /// The bullets behind a red "New failures" verdict, inline so the regressed
    /// projects can be judged without opening the XLSX.
    fn new_failure_bullets(stats: &[SuiteStats]) -> Vec<String> {
        let mut bullets = Vec::new();
        for suite in stats {
            let ranked = suite.failure_regressions.ranked();
            let truncated = Truncated::new(ranked.as_slice());
            for regression in truncated.shown {
                bullets.push(format!(
                    "{}: `{}` [{}] {} failures {} → {}",
                    suite.label,
                    regression.label,
                    regression.mode,
                    regression.kind,
                    regression.main,
                    regression.pr
                ));
            }
            bullets.extend(truncated.more_bullet(suite.report_file.as_str()));
        }
        bullets
    }

    /// The PR-vs-baseline size rows: one row per suite and pipeline, each cell
    /// the PR total against a released baseline.
    fn baseline_rows(stats: &[SuiteStats]) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        for suite in stats
            .iter()
            .filter(|suite| !suite.baseline_pairs.is_empty())
        {
            let pipelines: BTreeSet<Pipeline> = suite
                .baseline_pairs
                .keys()
                .map(|(_, pipeline)| *pipeline)
                .collect();
            for pipeline in pipelines {
                let versus = |role: Role| -> String {
                    suite
                        .baseline_pairs
                        .get(&(role, pipeline))
                        .and_then(|pair| crate::utils::relative_percent(pair.pr, pair.baseline))
                        .map(crate::utils::percent)
                        .unwrap_or_else(|| "—".to_owned())
                };
                rows.push(vec![
                    suite.suite_cell(),
                    pipeline.to_string(),
                    versus(Role::Solc),
                    versus(Role::Latest),
                ]);
            }
        }
        rows
    }
}
