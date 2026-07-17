//!
//! The full summary comment: every string the markdown template interpolates,
//! precomputed from the per-suite statistics.
//!
//! The comment's shape — section order, headers, table pipes, bullet and
//! blank-line discipline — lives in `templates/summary.md`; everything the
//! template interpolates is a string precomputed here. The boundary rule: the
//! template may test presence (`if let`, `is_empty`), never magnitude; anything
//! that formats a value is Rust.
//!

pub mod listing_section;

use std::collections::BTreeSet;

use askama::Template;

use crate::output::summary::compile_view::CompileView;
use crate::output::summary::failure_verdict::FailureVerdict;
use crate::output::summary::health_issue::HealthIssue;
use crate::output::summary::output_verdict::OutputVerdict;
use crate::output::summary::suite_row::SuiteRow;
use crate::output::summary::suite_stats::SuiteStats;
use crate::output::summary::truncated::Truncated;
use crate::role::Role;
use crate::utils::agreeing;
use crate::utils::count_noun;
use crate::utils::percent;
use crate::utils::relative_percent;

use self::listing_section::ListingSection;

///
/// The full summary comment.
///
#[derive(Template)]
#[template(path = "summary.md", escape = "none")]
pub struct SummaryTemplate {
    full_matrix: bool,
    output_line: String,
    failures_line: String,
    health_lines: Vec<String>,
    warn_lines: Vec<String>,
    suite_rows: Vec<SuiteRow>,
    new_failure_bullets: Vec<String>,
    mover_sections: Vec<ListingSection>,
    compile: Option<CompileView>,
    baseline_rows: Vec<Vec<String>>,
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
            full_matrix: stats.iter().any(|s| s.has_baselines),
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
                "⚠️ **No baseline** — {} {} no `main` counterpart; {} failures are not compared.",
                unbaselined.join("; "),
                agreeing(unbaselined_runs as u64, "has", "have"),
                agreeing(unbaselined_runs as u64, "its", "their")
            ));
        }
        if !main_only.is_empty() {
            warn_lines.push(format!(
                "⚠️ **Missing on PR** — {} {} only on `main`; the comparison set shrank.",
                main_only.join("; "),
                agreeing(main_only_runs as u64, "exists", "exist")
            ));
        }
        (lines, warn_lines)
    }

    /// One suite's share of an unpaired-runs warning line — the same wording
    /// for the no-baseline and main-only directions.
    fn unpaired_runs_part(label: &str, runs: usize, failures: usize) -> String {
        format!(
            "{label}: {} ({})",
            count_noun(runs as u64, "run"),
            count_noun(failures as u64, "failure")
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
        for s in stats {
            let ranked = s.failure_regressions.ranked();
            let truncated = Truncated::new(ranked.as_slice());
            for regression in truncated.shown {
                bullets.push(format!(
                    "{}: `{}` [{}] {} failures {} → {}",
                    s.label,
                    regression.label,
                    regression.mode,
                    regression.kind,
                    regression.main,
                    regression.pr
                ));
            }
            bullets.extend(truncated.more_bullet(s.report_file.as_str()));
        }
        bullets
    }

    fn baseline_rows(stats: &[SuiteStats]) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        for s in stats.iter().filter(|s| !s.baseline_pairs.is_empty()) {
            let pipelines: BTreeSet<&String> = s
                .baseline_pairs
                .keys()
                .map(|(_, pipeline)| pipeline)
                .collect();
            for pipeline in pipelines {
                let vs = |role: Role| -> String {
                    s.baseline_pairs
                        .get(&(role, pipeline.clone()))
                        .and_then(|pair| relative_percent(pair.pr, pair.baseline))
                        .map(percent)
                        .unwrap_or_else(|| "—".to_owned())
                };
                rows.push(vec![
                    s.suite_cell(),
                    pipeline.clone(),
                    vs(Role::Solc),
                    vs(Role::Latest),
                ]);
            }
        }
        rows
    }
}
