//!
//! The markdown rendering of the integration summary.
//!
//! The comment's shape — section order, headers, table pipes, bullet and
//! blank-line discipline — lives in `templates/summary.md`; everything the
//! template interpolates is a string precomputed here. The boundary rule:
//! the template may test presence (`if let`, `is_empty`), never magnitude;
//! anything that formats a value is Rust.
//!

use std::collections::BTreeSet;

use askama::Template;

use crate::utils::agreeing;
use crate::utils::commas;
use crate::utils::count_noun;
use crate::utils::median;
use crate::utils::percent;
use crate::utils::relative_percent;
use crate::utils::signed_commas;

use super::SuiteOutcome;
use super::compile_aggregate::CompileAggregate;
use super::diff_counter::DiffCounter;
use super::failure_verdict::FailureVerdict;
use super::health_issue::HealthIssue;
use super::output_verdict::OutputVerdict;
use super::suite_stats::SuiteStats;
use super::toolchain::Role;

/// A compile-time move on one project large enough to surface individually.
const COMPILE_TIME_PROJECT_THRESHOLD_PERCENT: f64 = 15.0;
/// A suite-aggregate compile-time move large enough to highlight.
const COMPILE_TIME_SUITE_THRESHOLD_PERCENT: f64 = 5.0;
/// Cap on individually-listed items (outliers, movers, new failures) before "+N more".
const MAX_LISTED: usize = 5;
/// Jitter medians below this render as "<0.1%": the floor under which the
/// one-decimal display precision would round to a bare 0.0%.
const JITTER_MEDIAN_FLOOR_PERCENT: f64 = 0.05;

impl DiffCounter {
    /// One comparison column's table cell; the byte delta rides along for
    /// size cells.
    fn cell(&self, delta_suffix: bool) -> String {
        if !self.collected() {
            return "⚪ not collected".to_owned();
        }
        if self.diffs == 0 {
            return format!("✅ 0 of {}", commas(self.cells));
        }
        let delta = if delta_suffix {
            format!(" ({} B)", signed_commas(self.delta))
        } else {
            String::new()
        };
        format!("⚠️ {} of {}{delta}", commas(self.diffs), commas(self.cells))
    }
}

impl SuiteStats {
    fn failures_cell(&self) -> String {
        let unbaselined = match self.unbaselined_failures {
            0 => String::new(),
            n => format!(", ⚪ {} unbaselined", commas(n as u64)),
        };
        if self.paired_runs == 0 {
            return format!("⚪ not compared{unbaselined}");
        }
        let pre = match self.baseline_failures() {
            0 => String::new(),
            n => format!(" ({} pre-existing)", commas(n as u64)),
        };
        if self.new_failures() == 0 {
            format!("✅ 0{pre}{unbaselined}")
        } else {
            format!(
                "❌ {}{pre}{unbaselined}",
                new_failure_kinds(self.new_build_failures, self.new_test_failures)
            )
        }
    }

    fn gas_cell(&self) -> String {
        if !self.gas.collected() {
            return "⚪ not collected".to_owned();
        }
        if !self.gas_is_gate {
            let mut parts = Vec::new();
            if !self.gas_jitter_percents.is_empty() {
                let med = match median(&self.gas_jitter_percents) {
                    Some(med) if med >= JITTER_MEDIAN_FLOOR_PERCENT => format!("{med:.1}%"),
                    _ => "<0.1%".to_owned(),
                };
                parts.push(format!(
                    "jitter {} of {}, median {med}",
                    commas(self.gas_jitter_percents.len() as u64),
                    commas(self.gas.cells)
                ));
            }
            if self.gas_one_sided > 0 {
                parts.push(format!("{} one-sided", commas(self.gas_one_sided)));
            }
            if parts.is_empty() {
                return "⚪ no jitter (not gated)".to_owned();
            }
            return format!("⚪ {} (not gated)", parts.join("; "));
        }
        self.gas.cell(false)
    }

    fn size_cell(&self) -> String {
        if self.size_one_sided > 0 {
            let one_sided = format!("⚪ {} one-sided", commas(self.size_one_sided));
            if !self.size.collected() {
                return one_sided;
            }
            return format!("{}, {one_sided}", self.size.cell(true));
        }
        self.size.cell(true)
    }

    fn suite_cell(&self) -> String {
        if self.project_count > 1 {
            format!("{} · {} proj", self.label, self.project_count)
        } else {
            self.label.clone()
        }
    }

    fn report_cell(&self) -> String {
        match self.report_url.as_deref() {
            Some(url) => format!("[{} ↓]({url})", self.report_file),
            None => "—".to_owned(),
        }
    }
}

/// One row of the results table, a cell per column.
struct SuiteRow {
    suite: String,
    failures: String,
    size: String,
    gas: String,
    report: String,
}

/// One bulleted listing under a bold heading, already truncated: a "+N more"
/// pointer is its last bullet.
struct ListingSection {
    heading: String,
    bullets: Vec<String>,
}

/// A listing capped at `MAX_LISTED`: what to show, and how many the cap left
/// out. The cap, the arithmetic, and the overflow wording live here, so the
/// listings cannot drift into several spellings of one rule.
struct Truncated<'a, T> {
    shown: &'a [T],
    extra: usize,
}

impl<'a, T> Truncated<'a, T> {
    /// Caps a listing at `MAX_LISTED`, keeping the leading items and counting
    /// the rest as overflow. Callers that want the surviving items to be the
    /// most significant rank the slice before passing it.
    fn new(items: &'a [T]) -> Self {
        Self {
            shown: &items[..items.len().min(MAX_LISTED)],
            extra: items.len().saturating_sub(MAX_LISTED),
        }
    }

    /// The bullet closing a truncated listing, pointing at the full report.
    fn more_bullet(&self, report_file: &str) -> Option<String> {
        (self.extra > 0).then(|| format!("+{} more — see {report_file}", self.extra))
    }

    /// The suffix closing a truncated inline list.
    fn more_suffix(&self) -> String {
        match self.extra {
            0 => String::new(),
            extra => format!(" (+{extra} more)"),
        }
    }
}

/// The compile-time table and its threshold verdict lines; the columns are
/// data-driven, so the header repeats per pipeline.
struct CompileView {
    pipelines: Vec<String>,
    rows: Vec<Vec<String>>,
    conclusion_line: Option<String>,
    outliers_line: Option<String>,
}

///
/// The full summary comment.
///
#[derive(Template)]
#[template(path = "summary.md", escape = "none")]
struct SummaryTemplate {
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
    ///
    /// Everything the template interpolates, precomputed from the per-suite
    /// statistics.
    ///
    fn from_stats(stats: &[SuiteStats]) -> Self {
        let (health_lines, warn_lines) = health_lines(stats);
        Self {
            full_matrix: stats.iter().any(|s| s.has_baselines),
            output_line: OutputVerdict::from_stats(stats).line(),
            failures_line: FailureVerdict::from_stats(stats).line(),
            health_lines,
            warn_lines,
            suite_rows: stats.iter().map(SuiteStats::row).collect(),
            new_failure_bullets: new_failure_bullets(stats),
            mover_sections: ListingSection::from_stats(stats),
            compile: CompileView::from_stats(stats),
            baseline_rows: baseline_rows(stats),
        }
    }
}

///
/// Renders the full summary comment for the given per-suite statistics.
///
pub(crate) fn render_summary(stats: &[SuiteStats]) -> String {
    SummaryTemplate::from_stats(stats)
        .render()
        .expect("template rendering writes to a String")
}

impl OutputVerdict {
    /// The output-invariance verdict line.
    fn line(self) -> String {
        match self {
            Self::NoData => {
                "⚪ **No output data** — no size or gated-gas comparison had a `main` counterpart \
                 to compare against."
                    .to_owned()
            }
            Self::Preserving {
                size_cells,
                gated_gas_cells,
                gas_label,
            } => {
                let mut clauses = Vec::new();
                if size_cells > 0 {
                    clauses.push(format!(
                        "bytecode size identical ({})",
                        count_noun(size_cells, "comparison")
                    ));
                }
                if gated_gas_cells > 0 {
                    clauses.push(format!(
                        "{gas_label} gas identical ({})",
                        commas(gated_gas_cells)
                    ));
                }
                format!("✅ **Output-preserving** — {}.", clauses.join(", "))
            }
            Self::Changed { size, gas } => {
                let mut parts = Vec::new();
                if let Some(size) = size {
                    parts.push(format!(
                        "{} of {} {} ({} B total)",
                        commas(size.diffs),
                        count_noun(size.cells, "size comparison"),
                        agreeing(size.diffs, "differs", "differ"),
                        signed_commas(size.delta_bytes)
                    ));
                }
                if let Some(gas) = gas {
                    parts.push(format!(
                        "{} of {} {}",
                        commas(gas.diffs),
                        count_noun(gas.cells, format!("{} gas comparison", gas.label).as_str()),
                        agreeing(gas.diffs, "differs", "differ")
                    ));
                }
                format!(
                    "⚠️ **Output changed** — {}. If this PR is meant to be output-preserving, \
                     investigate before merging.",
                    parts.join("; ")
                )
            }
        }
    }
}

impl FailureVerdict {
    /// The failure-regression verdict line.
    fn line(self) -> String {
        match self {
            Self::NoData => {
                "⚪ **No failure data** — no PR run had a `main` counterpart to compare against."
                    .to_owned()
            }
            Self::Clean { pre_existing } if pre_existing.is_empty() => {
                "✅ **No new failures**.".to_owned()
            }
            Self::Clean { pre_existing } => {
                let pre: Vec<String> = pre_existing
                    .iter()
                    .map(|(label, count)| format!("{label}'s {}", commas(*count as u64)))
                    .collect();
                format!(
                    "✅ **No new failures** — {} {} already present on `main`.",
                    pre.join(" / "),
                    agreeing(
                        pre_existing.iter().map(|(_, count)| *count as u64).sum(),
                        "failure",
                        "failures"
                    )
                )
            }
            Self::Regressed { suites } => {
                let parts: Vec<String> = suites
                    .iter()
                    .map(|suite| {
                        format!(
                            "{}: {}",
                            suite.label,
                            new_failure_kinds(suite.new_build, suite.new_test)
                        )
                    })
                    .collect();
                format!("❌ **New failures** — {}.", parts.join("; "))
            }
        }
    }
}

/// One suite's share of an unpaired-runs warning line — the same wording for
/// the no-baseline and main-only directions.
fn unpaired_runs_part(label: &str, runs: usize, failures: usize) -> String {
    format!(
        "{label}: {} ({})",
        count_noun(runs as u64, "run"),
        count_noun(failures as u64, "failure")
    )
}

/// Backtick-quoted mode strings, capped at `MAX_LISTED` with a "+N more".
fn truncated_mode_list(modes: &[String]) -> String {
    let truncated = Truncated::new(modes);
    let shown: Vec<String> = truncated
        .shown
        .iter()
        .map(|mode| format!("`{mode}`"))
        .collect();
    format!("{}{}", shown.join(", "), truncated.more_suffix())
}

/// The "+N build, +N test" list — one wording shared by the verdict line and
/// the table cell.
fn new_failure_kinds(build: usize, test: usize) -> String {
    let mut kinds = Vec::new();
    if build > 0 {
        kinds.push(format!("+{} build", commas(build as u64)));
    }
    if test > 0 {
        kinds.push(format!("+{} test", commas(test as u64)));
    }
    kinds.join(", ")
}

/// The harness-health lines, plus the aggregated no-baseline line that closes
/// the verdict block.
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
                    truncated_mode_list(&modes)
                ));
            }
            HealthIssue::UnrecognizedPipelines { label, modes } => {
                lines.push(format!(
                    "❌ **Harness error** — {label}: no recognized pipeline token in: {}.",
                    truncated_mode_list(&modes)
                ));
            }
            HealthIssue::Unbaselined {
                label,
                runs,
                failures,
            } => {
                unbaselined_runs += runs;
                unbaselined.push(unpaired_runs_part(&label, runs, failures));
            }
            HealthIssue::MainOnly {
                label,
                runs,
                failures,
            } => {
                main_only_runs += runs;
                main_only.push(unpaired_runs_part(&label, runs, failures));
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

impl SuiteStats {
    /// The suite's row in the results table. A suite with no comparable data
    /// dashes its measurement columns rather than rendering a zero.
    fn row(&self) -> SuiteRow {
        let dashed = |failures: &str| SuiteRow {
            suite: self.label.clone(),
            failures: failures.to_owned(),
            size: "—".to_owned(),
            gas: "—".to_owned(),
            report: self.report_cell(),
        };
        if self.outcome == SuiteOutcome::Skipped {
            return dashed("⚪ did not run");
        }
        if !self.available {
            return dashed("❌ no report — suite errored");
        }
        if self.is_empty_report() {
            return dashed("❌ empty report");
        }
        if self.classification_failed() {
            return dashed("❌ unrecognized toolchain naming");
        }
        SuiteRow {
            suite: self.suite_cell(),
            failures: self.failures_cell(),
            size: self.size_cell(),
            gas: self.gas_cell(),
            report: self.report_cell(),
        }
    }
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

impl ListingSection {
    /// The listings behind an "Output changed" verdict, inline — a bytecode
    /// size change means semantics possibly changed, so it is never folded
    /// away.
    fn from_stats(stats: &[SuiteStats]) -> Vec<Self> {
        let mut sections = Vec::new();
        for s in stats {
            for (title, unit, movers) in [
                ("largest size changes", " B", &s.top_size_movers),
                ("largest gas changes", "", &s.top_gas_movers),
            ] {
                if movers.is_empty() {
                    continue;
                }
                let ranked = movers.ranked();
                let truncated = Truncated::new(ranked.as_slice());
                let mut bullets: Vec<String> = truncated
                    .shown
                    .iter()
                    .map(|m| {
                        let pct = match relative_percent(m.pr, m.main) {
                            Some(pct) => format!(" ({})", percent(pct)),
                            None => String::new(),
                        };
                        format!(
                            "`{}` [{}] {} → {}{unit}{pct}",
                            m.label,
                            m.mode,
                            commas(m.main),
                            commas(m.pr)
                        )
                    })
                    .collect();
                bullets.extend(truncated.more_bullet(s.report_file.as_str()));
                sections.push(Self {
                    heading: format!("{} — {title}", s.label),
                    bullets,
                });
            }
        }
        sections
    }
}

impl CompileView {
    /// The compile-time table and its verdict lines, or `None` when no suite
    /// collected compile times at all.
    fn from_stats(stats: &[SuiteStats]) -> Option<Self> {
        let with_ct: Vec<&SuiteStats> = stats.iter().filter(|s| !s.compile.is_empty()).collect();
        if with_ct.is_empty() {
            return None;
        }
        let pipelines: Vec<String> = with_ct
            .iter()
            .flat_map(|s| s.compile.keys())
            .map(String::clone)
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect();

        let mut any_paired = false;
        let mut any_suite_flag = false;
        let mut outlier_entries: Vec<(String, String, f64)> = Vec::new();
        let mut rows = Vec::new();
        for s in &with_ct {
            let mut row = vec![s.suite_cell()];
            for pipeline in pipelines.iter() {
                let paired = s.compile.get(pipeline).and_then(|agg| {
                    relative_percent(agg.pr_total_ms, agg.main_total_ms).map(|pct| (agg, pct))
                });
                row.push(match paired {
                    Some((agg, pct)) => {
                        any_paired = true;
                        let (cell, flagged) = agg.cell(pct);
                        any_suite_flag |= flagged;
                        cell
                    }
                    None => "—".to_owned(),
                });
                if let Some(agg) = s.compile.get(pipeline) {
                    for (project, pct) in agg.per_project.iter() {
                        if pct.abs() >= COMPILE_TIME_PROJECT_THRESHOLD_PERCENT {
                            outlier_entries.push((project.clone(), pipeline.clone(), *pct));
                        }
                    }
                }
            }
            rows.push(row);
        }

        let conclusion_line = if !any_paired {
            Some(
                "_No paired compile-time data — no pipeline was measured on both `main` and the \
                 PR._"
                    .to_owned(),
            )
        } else if outlier_entries.is_empty() && !any_suite_flag {
            Some(format!(
                "_Within noise — no suite ≥ {}%, no project ≥ {}%._",
                COMPILE_TIME_SUITE_THRESHOLD_PERCENT as u64,
                COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64
            ))
        } else {
            None
        };
        let outliers_line =
            (!outlier_entries.is_empty()).then(|| outliers_line(&mut outlier_entries));
        Some(Self {
            pipelines,
            rows,
            conclusion_line,
            outliers_line,
        })
    }
}

impl CompileAggregate {
    /// One aggregate/median compile cell, and whether it crossed the suite
    /// threshold. Both directions defeat "within noise", but only a slowdown
    /// gets the siren — a large improvement is signal, not an alarm.
    fn cell(&self, pct: f64) -> (String, bool) {
        let (aggregate, flagged) = if pct >= COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
            (format!("⚠️ **{}**", percent(pct)), true)
        } else if pct <= -COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
            (format!("**{}**", percent(pct)), true)
        } else {
            (percent(pct), false)
        };
        let project_pcts: Vec<f64> = self.per_project.iter().map(|(_, pct)| *pct).collect();
        match median(project_pcts.as_slice()) {
            Some(med) => (format!("{aggregate} / {}", percent(med)), flagged),
            None => (aggregate, flagged),
        }
    }
}

fn outliers_line(outliers: &mut [(String, String, f64)]) -> String {
    outliers.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
    let siren = if outliers.iter().any(|(_, _, pct)| *pct > 0.0) {
        "⚠️ "
    } else {
        ""
    };
    let truncated = Truncated::new(outliers);
    let shown: Vec<String> = truncated
        .shown
        .iter()
        .map(|(project, pipeline, pct)| format!("`{project}` {pipeline} **{}**", percent(*pct)))
        .collect();
    format!(
        "{siren}**Project outliers (≥{}%):** {}{}",
        COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64,
        shown.join(" · "),
        truncated.more_suffix()
    )
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
