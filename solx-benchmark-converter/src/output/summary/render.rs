//!
//! The markdown rendering of the integration summary (askama spike).
//!
//! The comment's structure lives in `templates/summary.md`; this file holds
//! the cell-level formatting the template calls back into, the custom
//! filters, and the view models precomputed for the data-driven tables.
//!

use std::collections::BTreeSet;

use askama::Template;

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
            let mut kinds = Vec::new();
            if self.new_build_failures > 0 {
                kinds.push(format!("+{} build", commas(self.new_build_failures as u64)));
            }
            if self.new_test_failures > 0 {
                kinds.push(format!("+{} test", commas(self.new_test_failures as u64)));
            }
            format!("❌ {}{pre}{unbaselined}", kinds.join(", "))
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

/// One "largest changes" listing, ranked; the template truncates it.
struct MoverSection {
    suite_label: String,
    title: &'static str,
    unit: &'static str,
    movers: Vec<MoverRow>,
    report_file: String,
}

/// One mover, with the raw numbers so the template formats them.
struct MoverRow {
    label: String,
    mode: String,
    main: u64,
    pr: u64,
}

/// The compile-time table, precomputed because its columns are data-driven.
struct CompileView {
    pipelines: Vec<String>,
    rows: Vec<Vec<String>>,
    within_noise: bool,
    outliers: Option<String>,
}

///
/// The full summary comment.
///
#[derive(Template)]
#[template(path = "summary.md", escape = "none")]
struct SummaryTemplate<'a> {
    full_matrix: bool,
    output: OutputVerdict,
    failures: FailureVerdict,
    issues: Vec<HealthIssue>,
    unbaselined: Vec<String>,
    stats: &'a [SuiteStats],
    has_new_failures: bool,
    mover_sections: Vec<MoverSection>,
    compile: Option<CompileView>,
    baseline_rows: Vec<Vec<String>>,
}

///
/// Renders the full summary comment for the given per-suite statistics.
///
pub(crate) fn render_summary(stats: &[SuiteStats]) -> String {
    let issues = health_issues(stats);
    let unbaselined: Vec<String> = issues
        .iter()
        .filter_map(|issue| match issue {
            HealthIssue::Unbaselined {
                label,
                runs,
                failures,
            } => Some(format!(
                "{label}: {runs} runs ({} failures)",
                commas(*failures as u64)
            )),
            _ => None,
        })
        .collect();
    let full_matrix = stats.iter().any(|s| s.has_baselines);
    SummaryTemplate {
        full_matrix,
        output: output_verdict(stats),
        failures: failure_verdict(stats),
        issues,
        unbaselined,
        stats,
        has_new_failures: stats.iter().any(|s| !s.failure_regressions.is_empty()),
        mover_sections: build_mover_sections(stats),
        compile: build_compile_view(stats),
        baseline_rows: if full_matrix {
            build_baseline_rows(stats)
        } else {
            Vec::new()
        },
    }
    .render()
    .expect("template rendering writes to a String")
}

fn build_mover_sections(stats: &[SuiteStats]) -> Vec<MoverSection> {
    let mut sections = Vec::new();
    for s in stats {
        for (title, unit, movers) in [
            ("largest size changes", " B", &s.top_size_movers),
            ("largest gas changes", "", &s.top_gas_movers),
        ] {
            if movers.is_empty() {
                continue;
            }
            sections.push(MoverSection {
                suite_label: s.label.clone(),
                title,
                unit,
                movers: movers
                    .ranked()
                    .into_iter()
                    .map(|m| MoverRow {
                        label: m.label.clone(),
                        mode: m.mode.clone(),
                        main: m.main,
                        pr: m.pr,
                    })
                    .collect(),
                report_file: s.report_file.clone(),
            });
        }
    }
    sections
}

fn build_compile_view(stats: &[SuiteStats]) -> Option<CompileView> {
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

    let within_noise = outlier_entries.is_empty() && !any_suite_flag;
    let outliers = (!outlier_entries.is_empty()).then(|| outliers_line(&mut outlier_entries));
    Some(CompileView {
        pipelines,
        rows,
        within_noise,
        outliers,
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

fn build_baseline_rows(stats: &[SuiteStats]) -> Vec<Vec<String>> {
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

/// Custom template filters.
mod filters {
    /// Thousands separators for u64 counts.
    pub fn commas(n: &u64, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(super::commas(*n))
    }

    /// Thousands separators for usize counts.
    pub fn commas_usize(n: &usize, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(super::commas(*n as u64))
    }

    /// The " (+x.x%)" suffix for a mover row, empty on a zero base.
    pub fn rel_suffix(pr: &u64, _: &dyn askama::Values, main: &u64) -> askama::Result<String> {
        Ok(match super::relative_percent(*pr, *main) {
            Some(pct) => format!(" ({})", super::percent(pct)),
            None => String::new(),
        })
    }

    /// A float threshold rendered as its integer floor.
    pub fn floor_u64(f: f64, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("{}", f as u64))
    }
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
