//!
//! The markdown rendering of the integration summary.
//!
//! Everything here turns already-decided verdicts and already-computed
//! statistics into text; no thresholds or comparisons are evaluated in this
//! file except the presentation caps below.
//!

use std::collections::BTreeSet;
use std::fmt::Write;

use super::stats::SuiteStats;
use super::stats::median;
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

    fn size_cell(&self) -> String {
        if !self.size.collected() {
            return "⚪ not collected".to_owned();
        }
        if self.size.diffs == 0 {
            format!("✅ 0 of {}", commas(self.size.cells))
        } else {
            format!(
                "⚠️ {} of {} ({:+} B)",
                commas(self.size.diffs),
                commas(self.size.cells),
                self.size.delta
            )
        }
    }

    fn gas_cell(&self) -> String {
        if !self.gas.collected() {
            return "⚪ not collected".to_owned();
        }
        if !self.gas_is_gate {
            if self.gas.diffs == 0 {
                return "⚪ no jitter (not gated)".to_owned();
            }
            let med = match median(&self.gas_jitter_percents) {
                Some(med) if med >= 0.05 => format!("{med:.1}%"),
                _ => "<0.1%".to_owned(),
            };
            return format!(
                "⚪ jitter {} of {}, median {med} (not gated)",
                commas(self.gas.diffs),
                commas(self.gas.cells)
            );
        }
        if self.gas.diffs == 0 {
            format!("✅ 0 of {}", commas(self.gas.cells))
        } else {
            format!(
                "⚠️ {} of {}",
                commas(self.gas.diffs),
                commas(self.gas.cells)
            )
        }
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

///
/// The verdict lines: output invariance, new failures, and harness health —
/// three independent signals decided in `verdict.rs`, each stated with its
/// numbers.
///
pub(crate) fn render_verdict(out: &mut String, stats: &[SuiteStats]) {
    match output_verdict(stats) {
        OutputVerdict::NoData => {
            let _ = writeln!(
                out,
                "⚪ **No output data** — no size or gated-gas comparisons were collected."
            );
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
            let _ = writeln!(out, "✅ **Output-preserving** — {}.", clauses.join(", "));
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
            let _ = writeln!(
                out,
                "⚠️ **Output changed** — {}. If this PR is meant to be output-preserving, investigate before merging.",
                parts.join("; ")
            );
        }
    }

    match failure_verdict(stats) {
        FailureVerdict::Clean { pre_existing } if pre_existing.is_empty() => {
            let _ = writeln!(out, "✅ **No new failures**.");
        }
        FailureVerdict::Clean { pre_existing } => {
            let pre: Vec<String> = pre_existing
                .iter()
                .map(|(label, count)| format!("{label}'s {}", commas(*count as u64)))
                .collect();
            let _ = writeln!(
                out,
                "✅ **No new failures** — {} failures already present on `main`.",
                pre.join(" / ")
            );
        }
        FailureVerdict::Regressed { suites } => {
            let parts: Vec<String> = suites
                .iter()
                .map(|suite| {
                    let mut kinds = Vec::new();
                    if suite.new_build > 0 {
                        kinds.push(format!("+{} build", commas(suite.new_build as u64)));
                    }
                    if suite.new_test > 0 {
                        kinds.push(format!("+{} test", commas(suite.new_test as u64)));
                    }
                    format!("{}: {}", suite.label, kinds.join(", "))
                })
                .collect();
            let _ = writeln!(out, "❌ **New failures** — {}.", parts.join("; "));
        }
    }

    let mut unbaselined = Vec::new();
    for issue in health_issues(stats) {
        match issue {
            HealthIssue::SuiteErrored { label } => {
                let _ = writeln!(
                    out,
                    "❌ **Suite errored** — {label} produced no usable report."
                );
            }
            HealthIssue::UnrecognizedToolchains { label } => {
                let _ = writeln!(
                    out,
                    "❌ **Harness error** — {label}: benchmark data matched no recognized toolchain naming."
                );
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
                let _ = writeln!(
                    out,
                    "❌ **Harness error** — {label}: runs matched no declared toolchain: {}{more}.",
                    shown.join(", ")
                );
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
        }
    }
    if !unbaselined.is_empty() {
        let _ = writeln!(
            out,
            "⚠️ **No baseline** — {} have no `main` counterpart; their failures are not compared.",
            unbaselined.join("; ")
        );
    }
    let _ = writeln!(out);
}

pub(crate) fn render_results_table(out: &mut String, stats: &[SuiteStats]) {
    let _ = writeln!(out, "| Suite | New failures | Size Δ | Gas Δ | Report |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for s in stats {
        if !s.available {
            let _ = writeln!(
                out,
                "| {} | ❌ no report — suite errored | — | — | {} |",
                s.label,
                s.report_cell()
            );
            continue;
        }
        if s.classification_failed() {
            let _ = writeln!(
                out,
                "| {} | ❌ unrecognized toolchain naming | — | — | {} |",
                s.label,
                s.report_cell()
            );
            continue;
        }
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            s.suite_cell(),
            s.failures_cell(),
            s.size_cell(),
            s.gas_cell(),
            s.report_cell(),
        );
    }
}

///
/// The rows behind a red "New failures" verdict, inline so the regressed
/// projects can be judged without opening the XLSX.
///
pub(crate) fn render_new_failures(out: &mut String, stats: &[SuiteStats]) {
    if stats.iter().all(|s| s.failure_regressions.is_empty()) {
        return;
    }
    let _ = writeln!(out, "\n**New failures (PR vs `main`):**\n");
    for s in stats {
        for regression in s.failure_regressions.iter().take(MAX_LISTED) {
            let _ = writeln!(
                out,
                "- {}: `{}` [{}] {} failures {} → {}",
                s.label,
                regression.label,
                regression.mode,
                regression.kind,
                regression.main,
                regression.pr
            );
        }
        let extra = s.failure_regressions.len().saturating_sub(MAX_LISTED);
        if extra > 0 {
            let _ = writeln!(out, "- +{extra} more — see {}", s.report_file);
        }
    }
}

///
/// The rows behind an "Output changed" verdict, inline — a bytecode size
/// change means semantics possibly changed, so it is never folded away.
///
pub(crate) fn render_output_changes(out: &mut String, stats: &[SuiteStats]) {
    for s in stats {
        for (title, unit, movers) in [
            ("largest size changes", " B", &s.top_size_movers),
            ("largest gas changes", "", &s.top_gas_movers),
        ] {
            if movers.is_empty() {
                continue;
            }
            let _ = writeln!(out, "\n**{} — {title}:**\n", s.label);
            for m in movers.ranked().into_iter().take(MAX_LISTED) {
                let pct = if m.main != 0 {
                    format!(
                        " ({})",
                        percent((m.pr as f64 - m.main as f64) / m.main as f64 * 100.0)
                    )
                } else {
                    String::new()
                };
                let _ = writeln!(
                    out,
                    "- `{}` [{}] {} → {}{unit}{pct}",
                    m.label,
                    m.mode,
                    commas(m.main),
                    commas(m.pr),
                );
            }
            let extra = movers.len().saturating_sub(MAX_LISTED);
            if extra > 0 {
                let _ = writeln!(out, "- +{extra} more — full list in {}", s.report_file);
            }
        }
    }
}

pub(crate) fn render_compile_time(out: &mut String, stats: &[SuiteStats]) {
    let with_ct: Vec<&SuiteStats> = stats.iter().filter(|s| !s.compile.is_empty()).collect();
    if with_ct.is_empty() {
        return;
    }

    // Columns come from the pipelines actually present so a new codegen
    // shows up instead of silently vanishing from the tripwire.
    let pipelines: Vec<&str> = with_ct
        .iter()
        .flat_map(|s| s.compile.keys())
        .map(String::as_str)
        .collect::<BTreeSet<&str>>()
        .into_iter()
        .collect();
    let _ = writeln!(
        out,
        "\n**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)\n"
    );
    let mut header = "| Suite |".to_owned();
    let mut divider = "|---|".to_owned();
    for pipeline in pipelines.iter() {
        let _ = write!(header, " {pipeline} (agg / median) |");
        divider.push_str("---|");
    }
    let _ = writeln!(out, "{header}");
    let _ = writeln!(out, "{divider}");

    let mut any_suite_flag = false;
    let mut outliers: Vec<(String, String, f64)> = Vec::new();
    for s in &with_ct {
        let mut row = format!("| {} |", s.suite_cell());
        for pipeline in pipelines.iter() {
            let cell = match s.compile.get(*pipeline) {
                Some(agg) if agg.main_total_ms != 0 => {
                    let pct = (agg.pr_total_ms as f64 - agg.main_total_ms as f64)
                        / agg.main_total_ms as f64
                        * 100.0;
                    let aggregate = if pct.abs() >= COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
                        any_suite_flag = true;
                        format!("⚠️ **{}**", percent(pct))
                    } else {
                        percent(pct)
                    };
                    let project_pcts: Vec<f64> =
                        agg.per_project.iter().map(|(_, pct)| *pct).collect();
                    match median(project_pcts.as_slice()) {
                        Some(med) => format!("{aggregate} / {}", percent(med)),
                        None => aggregate,
                    }
                }
                _ => "—".to_owned(),
            };
            let _ = write!(row, " {cell} |");
            if let Some(agg) = s.compile.get(*pipeline) {
                for (project, pct) in agg.per_project.iter() {
                    if pct.abs() >= COMPILE_TIME_PROJECT_THRESHOLD_PERCENT {
                        outliers.push((project.clone(), (*pipeline).to_owned(), *pct));
                    }
                }
            }
        }
        let _ = writeln!(out, "{row}");
    }

    if outliers.is_empty() && !any_suite_flag {
        let _ = writeln!(
            out,
            "\n_Within noise — no suite ≥ {}%, no project ≥ {}%._",
            COMPILE_TIME_SUITE_THRESHOLD_PERCENT as u64,
            COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64
        );
    }
    if !outliers.is_empty() {
        outliers.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
        let shown: Vec<String> = outliers
            .iter()
            .take(MAX_LISTED)
            .map(|(project, pipeline, pct)| format!("`{project}` {pipeline} **{}**", percent(*pct)))
            .collect();
        let mut line = format!(
            "\n⚠️ **Project outliers (>{}%):** {}",
            COMPILE_TIME_PROJECT_THRESHOLD_PERCENT as u64,
            shown.join(" · ")
        );
        if outliers.len() > MAX_LISTED {
            let _ = write!(line, " (+{} more)", outliers.len() - MAX_LISTED);
        }
        let _ = writeln!(out, "{line}");
    }
}

pub(crate) fn render_baselines(out: &mut String, stats: &[SuiteStats]) {
    let relevant: Vec<&SuiteStats> = stats
        .iter()
        .filter(|s| !s.baseline_pairs.is_empty())
        .collect();
    if relevant.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n**Bytecode size — PR vs baselines** (positive = PR larger; contracts built by both only)\n"
    );
    let _ = writeln!(out, "| Suite | Pipeline | vs solc | vs released solx |");
    let _ = writeln!(out, "|---|---|---|---|");
    for s in relevant {
        let pipelines: BTreeSet<&String> = s
            .baseline_pairs
            .keys()
            .map(|(_, pipeline)| pipeline)
            .collect();
        for pipeline in pipelines {
            let vs = |role: Role| -> String {
                match s.baseline_pairs.get(&(role, pipeline.clone())) {
                    Some(pair) if pair.baseline != 0 => percent(
                        (pair.pr as f64 - pair.baseline as f64) / pair.baseline as f64 * 100.0,
                    ),
                    _ => "—".to_owned(),
                }
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                s.suite_cell(),
                pipeline,
                vs(Role::Solc),
                vs(Role::Latest)
            );
        }
    }
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
