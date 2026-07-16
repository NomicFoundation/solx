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

use super::SuiteOutcome;
use super::stats::CompileAggregate;
use super::stats::DiffCounter;
use super::stats::SuiteStats;
use super::stats::median;
use super::stats::relative_percent;
use super::toolchain::Role;
use super::verdict::FailureVerdict;
use super::verdict::HealthIssue;
use super::verdict::OutputVerdict;
use super::verdict::failure_verdict;
use super::verdict::health_issues;
use super::verdict::output_verdict;

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
            format!(" ({:+} B)", self.delta)
        } else {
            String::new()
        };
        format!("⚠️ {} of {}{delta}", commas(self.diffs), commas(self.cells))
    }
}

impl SuiteStats {
    fn failures_cell(&self) -> String {
        let pre = match self.baseline_failures() {
            0 => String::new(),
            n => format!(" ({} pre-existing)", commas(n as u64)),
        };
        let unbaselined = match self.unbaselined_failures {
            0 => String::new(),
            n => format!(", ⚪ {} unbaselined", commas(n as u64)),
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
            // The count and the median come from the same population: pairs
            // with a `main` percentage. One-sided pairs are stated apart, so
            // an unbounded 0 → N addition is never averaged into "<0.1%".
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
            if self.gas_diffs_without_main > 0 {
                parts.push(format!(
                    "{} without `main` gas",
                    commas(self.gas_diffs_without_main)
                ));
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

/// The compile-time table and its threshold verdict lines; the columns are
/// data-driven, so the header repeats per pipeline.
struct CompileView {
    pipelines: Vec<String>,
    rows: Vec<Vec<String>>,
    within_noise_line: Option<String>,
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

///
/// Renders the full summary comment for the given per-suite statistics.
///
pub(crate) fn render_summary(stats: &[SuiteStats]) -> String {
    let (health_lines, warn_lines) = health_lines(stats);
    let full_matrix = stats.iter().any(|s| s.has_baselines);
    SummaryTemplate {
        full_matrix,
        output_line: output_line(output_verdict(stats)),
        failures_line: failures_line(failure_verdict(stats)),
        health_lines,
        warn_lines,
        suite_rows: stats.iter().map(suite_row).collect(),
        new_failure_bullets: new_failure_bullets(stats),
        mover_sections: mover_sections(stats),
        compile: compile_view(stats),
        baseline_rows: if full_matrix {
            baseline_rows(stats)
        } else {
            Vec::new()
        },
    }
    .render()
    .expect("template rendering writes to a String")
}

/// The output-invariance verdict line.
fn output_line(verdict: OutputVerdict) -> String {
    match verdict {
        OutputVerdict::NoData => {
            "⚪ **No output data** — no size or gated-gas comparisons were collected.".to_owned()
        }
        OutputVerdict::Preserving {
            size_cells,
            gated_gas_cells,
            gas_label,
        } => {
            let mut clauses = Vec::new();
            if size_cells > 0 {
                clauses.push(format!(
                    "bytecode size identical ({} comparisons)",
                    commas(size_cells)
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
        OutputVerdict::Changed { size, gas } => {
            let mut parts = Vec::new();
            if let Some(size) = size {
                parts.push(format!(
                    "{} of {} size comparisons differ ({:+} B total)",
                    commas(size.diffs),
                    commas(size.cells),
                    size.delta_bytes
                ));
            }
            if let Some(gas) = gas {
                parts.push(format!(
                    "{} of {} {} gas comparisons differ",
                    commas(gas.diffs),
                    commas(gas.cells),
                    gas.label
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

/// The failure-regression verdict line.
fn failures_line(verdict: FailureVerdict) -> String {
    match verdict {
        FailureVerdict::NoData => {
            "⚪ **No failure data** — no PR run had a `main` counterpart to compare against."
                .to_owned()
        }
        FailureVerdict::Clean { pre_existing } if pre_existing.is_empty() => {
            "✅ **No new failures**.".to_owned()
        }
        FailureVerdict::Clean { pre_existing } => {
            let pre: Vec<String> = pre_existing
                .iter()
                .map(|(label, count)| format!("{label}'s {}", commas(*count as u64)))
                .collect();
            format!(
                "✅ **No new failures** — {} failures already present on `main`.",
                pre.join(" / ")
            )
        }
        FailureVerdict::Regressed { suites } => {
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
    let mut main_only = Vec::new();
    for issue in health_issues(stats) {
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
                let shown: Vec<String> = modes
                    .iter()
                    .take(MAX_LISTED)
                    .map(|mode| format!("`{mode}`"))
                    .collect();
                let extra = modes.len().saturating_sub(MAX_LISTED);
                let more = if extra > 0 {
                    format!(" (+{extra} more)")
                } else {
                    String::new()
                };
                lines.push(format!(
                    "❌ **Harness error** — {label}: runs matched no declared toolchain: {}{more}.",
                    shown.join(", ")
                ));
            }
            HealthIssue::Unbaselined {
                label,
                runs,
                failures,
            } => {
                unbaselined.push(format!(
                    "{label}: {runs} runs ({} failures)",
                    commas(failures as u64)
                ));
            }
            HealthIssue::MainOnly {
                label,
                runs,
                failures,
            } => {
                main_only.push(format!(
                    "{label}: {runs} runs ({} failures)",
                    commas(failures as u64)
                ));
            }
        }
    }
    let mut warn_lines = Vec::new();
    if !unbaselined.is_empty() {
        warn_lines.push(format!(
            "⚠️ **No baseline** — {} have no `main` counterpart; their failures are not compared.",
            unbaselined.join("; ")
        ));
    }
    if !main_only.is_empty() {
        warn_lines.push(format!(
            "⚠️ **Missing on PR** — {} exist only on `main`; the comparison set shrank.",
            main_only.join("; ")
        ));
    }
    (lines, warn_lines)
}

fn suite_row(s: &SuiteStats) -> SuiteRow {
    let dashed = |failures: &str| SuiteRow {
        suite: s.label.clone(),
        failures: failures.to_owned(),
        size: "—".to_owned(),
        gas: "—".to_owned(),
        report: s.report_cell(),
    };
    if s.outcome == SuiteOutcome::Skipped {
        return dashed("⚪ did not run");
    }
    if !s.available {
        return dashed("❌ no report — suite errored");
    }
    if s.is_empty_report() {
        return dashed("❌ empty report");
    }
    if s.classification_failed() {
        return dashed("❌ unrecognized toolchain naming");
    }
    SuiteRow {
        suite: s.suite_cell(),
        failures: s.failures_cell(),
        size: s.size_cell(),
        gas: s.gas_cell(),
        report: s.report_cell(),
    }
}

/// The bullets behind a red "New failures" verdict, inline so the regressed
/// projects can be judged without opening the XLSX.
fn new_failure_bullets(stats: &[SuiteStats]) -> Vec<String> {
    let mut bullets = Vec::new();
    for s in stats {
        for regression in s.failure_regressions.iter().take(MAX_LISTED) {
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
        let extra = s.failure_regressions.len().saturating_sub(MAX_LISTED);
        if extra > 0 {
            bullets.push(format!("+{extra} more — see {}", s.report_file));
        }
    }
    bullets
}

/// The listings behind an "Output changed" verdict, inline — a bytecode size
/// change means semantics possibly changed, so it is never folded away.
fn mover_sections(stats: &[SuiteStats]) -> Vec<ListingSection> {
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
            let mut bullets: Vec<String> = ranked
                .iter()
                .take(MAX_LISTED)
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
            let extra = ranked.len().saturating_sub(MAX_LISTED);
            if extra > 0 {
                bullets.push(format!("+{extra} more — full list in {}", s.report_file));
            }
            sections.push(ListingSection {
                heading: format!("{} — {title}", s.label),
                bullets,
            });
        }
    }
    sections
}

fn compile_view(stats: &[SuiteStats]) -> Option<CompileView> {
    let with_ct: Vec<&SuiteStats> = stats.iter().filter(|s| !s.compile.is_empty()).collect();
    if with_ct.is_empty() {
        return None;
    }
    // Columns come from the pipelines actually present so a new codegen
    // shows up instead of silently vanishing from the tripwire.
    let pipelines: Vec<String> = with_ct
        .iter()
        .flat_map(|s| s.compile.keys())
        .map(String::clone)
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();

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
                Some((agg, pct)) => compile_cell(agg, pct, &mut any_suite_flag),
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

    let within_noise_line = (outlier_entries.is_empty() && !any_suite_flag).then(|| {
        format!(
            "_Within noise — no suite ≥ {}%, no project ≥ {}%._",
            COMPILE_TIME_SUITE_THRESHOLD_PERCENT as u64,
            COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64
        )
    });
    let outliers_line = (!outlier_entries.is_empty()).then(|| outliers_line(&mut outlier_entries));
    Some(CompileView {
        pipelines,
        rows,
        within_noise_line,
        outliers_line,
    })
}

/// One aggregate/median compile cell. Both directions defeat "within noise",
/// but only a slowdown gets the siren — a large improvement is signal, not
/// an alarm.
fn compile_cell(agg: &CompileAggregate, pct: f64, any_suite_flag: &mut bool) -> String {
    let aggregate = if pct >= COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
        *any_suite_flag = true;
        format!("⚠️ **{}**", percent(pct))
    } else if pct <= -COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
        *any_suite_flag = true;
        format!("**{}**", percent(pct))
    } else {
        percent(pct)
    };
    let project_pcts: Vec<f64> = agg.per_project.iter().map(|(_, pct)| *pct).collect();
    match median(project_pcts.as_slice()) {
        Some(med) => format!("{aggregate} / {}", percent(med)),
        None => aggregate,
    }
}

fn outliers_line(outliers: &mut [(String, String, f64)]) -> String {
    outliers.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
    let shown: Vec<String> = outliers
        .iter()
        .take(MAX_LISTED)
        .map(|(project, pipeline, pct)| format!("`{project}` {pipeline} **{}**", percent(*pct)))
        .collect();
    let siren = if outliers.iter().any(|(_, _, pct)| *pct > 0.0) {
        "⚠️ "
    } else {
        ""
    };
    let mut line = format!(
        "{siren}**Project outliers (≥{}%):** {}",
        COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64,
        shown.join(" · ")
    );
    if outliers.len() > MAX_LISTED {
        line.push_str(&format!(" (+{} more)", outliers.len() - MAX_LISTED));
    }
    line
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

/// Formats a percentage with a sign and one decimal.
fn percent(pct: f64) -> String {
    format!("{pct:+.1}%")
}

/// Formats an integer with thousands separators.
fn commas(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let bytes = digits.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && (bytes.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*byte as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::commas;

    #[test]
    fn commas_group_thousands() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(42), "42");
        assert_eq!(commas(47660), "47,660");
        assert_eq!(commas(101098), "101,098");
    }
}
