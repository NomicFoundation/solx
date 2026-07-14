//!
//! Markdown summary of an integration-test benchmark comparison.
//!
//! Renders the one-comment PR summary the integration workflow posts: the
//! correctness verdict (bytecode size everywhere + solx-tester gas), new
//! failures vs main, and a threshold-gated compile-time tripwire. The verdict
//! is computed here — the single source of truth shared by every suite —
//! instead of parsing the XLSX back offline.
//!
//! Golden tests pin full rendered comments under `output/summary/fixtures/`;
//! after an intended output change, regenerate them with
//! `UPDATE_SUMMARY_FIXTURES=1 cargo test -p solx-benchmark-converter`.
//!

mod stats;
mod toolchain;

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::benchmark::Benchmark;

use self::stats::SuiteStats;
use self::stats::median;
use self::toolchain::Role;

/// A compile-time move on one project large enough to surface individually.
const COMPILE_TIME_PROJECT_THRESHOLD_PERCENT: f64 = 15.0;
/// A suite-aggregate compile-time move large enough to highlight.
const COMPILE_TIME_SUITE_THRESHOLD_PERCENT: f64 = 5.0;
/// Cap on individually-listed items (outliers, movers, new failures) before "+N more".
const MAX_LISTED: usize = 5;

///
/// One suite (solx-tester / Foundry / Hardhat) fed into the summary.
///
pub struct SummarySuite {
    /// Human-readable suite name shown in the table.
    pub label: String,
    /// File name of the XLSX report inside the uploaded artifact, shown as
    /// link text and referenced by "+N more" pointers.
    pub report_file: String,
    /// The merged benchmark holding every toolchain's runs. `None` when the
    /// suite was expected but produced no report (it errored before writing) —
    /// rendered as an explicit failed row rather than silently dropped.
    pub benchmark: Option<Benchmark>,
    /// Artifact download URL for the XLSX report, if uploaded.
    pub report_url: Option<String>,
    /// Whether this suite's gas is deterministic and therefore gates
    /// correctness (true only for solx-tester's fixed REVM harness).
    pub gas_is_gate: bool,
}

///
/// Renders the full PR summary comment for the given suites.
///
pub fn render(suites: &[SummarySuite]) -> String {
    let stats: Vec<SuiteStats> = suites.iter().map(SuiteStats::from_suite).collect();

    let full_matrix = stats.iter().any(|s| s.has_baselines);

    let mut out = String::new();
    let mode = if full_matrix {
        "full matrix"
    } else {
        "standard"
    };
    let _ = writeln!(out, "### 🧪 Integration tests — {mode} · PR vs `main`\n");

    render_verdict(&mut out, &stats);
    render_results_table(&mut out, &stats);
    render_new_failures(&mut out, &stats);
    render_output_changes(&mut out, &stats);
    render_compile_time(&mut out, &stats);
    if full_matrix {
        render_baselines(&mut out, &stats);
    }

    let _ = writeln!(
        out,
        "\n---\n_Suites run the **release** solx binary. Foundry/Hardhat gas jitters run-to-run \
         (fuzz/invariant tests, CREATE-context deploys), so it never gates._"
    );

    out
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
/// three independent signals, each stated with its numbers.
///
fn render_verdict(out: &mut String, stats: &[SuiteStats]) {
    let size_cells: u64 = stats.iter().map(|s| s.size.cells).sum();
    let size_diffs: u64 = stats.iter().map(|s| s.size.diffs).sum();
    let size_delta: i128 = stats.iter().map(|s| s.size.delta).sum();
    let gated: Vec<&SuiteStats> = stats.iter().filter(|s| s.gas_is_gate).collect();
    let gated_gas_cells: u64 = gated.iter().map(|s| s.gas.cells).sum();
    let gated_gas_diffs: u64 = gated.iter().map(|s| s.gas.diffs).sum();
    let gas_label = gated
        .iter()
        .filter(|s| s.gas.collected())
        .map(|s| s.label.as_str())
        .collect::<Vec<_>>()
        .join(" / ");

    if size_diffs == 0 && gated_gas_diffs == 0 {
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
        if clauses.is_empty() {
            // Never a green checkmark over empty data.
            let _ = writeln!(
                out,
                "⚪ **No output data** — no size or gated-gas comparisons were collected."
            );
        } else {
            let _ = writeln!(out, "✅ **Output-preserving** — {}.", clauses.join(", "));
        }
    } else {
        let mut parts = Vec::new();
        if size_diffs > 0 {
            parts.push(format!(
                "{} of {} size comparisons differ ({:+} B total)",
                commas(size_diffs),
                commas(size_cells),
                size_delta
            ));
        }
        if gated_gas_diffs > 0 {
            parts.push(format!(
                "{} of {} {gas_label} gas comparisons differ",
                commas(gated_gas_diffs),
                commas(gated_gas_cells)
            ));
        }
        let _ = writeln!(
            out,
            "⚠️ **Output changed** — {}. If this PR is meant to be output-preserving, investigate before merging.",
            parts.join("; ")
        );
    }

    if stats.iter().all(|s| s.new_failures() == 0) {
        let pre: Vec<String> = stats
            .iter()
            .filter(|s| s.baseline_failures() > 0)
            .map(|s| format!("{}'s {}", s.label, commas(s.baseline_failures() as u64)))
            .collect();
        if pre.is_empty() {
            let _ = writeln!(out, "✅ **No new failures**.");
        } else {
            let _ = writeln!(
                out,
                "✅ **No new failures** — {} failures already present on `main`.",
                pre.join(" / ")
            );
        }
    } else {
        let parts: Vec<String> = stats
            .iter()
            .filter(|s| s.new_failures() > 0)
            .map(|s| {
                let mut kinds = Vec::new();
                if s.new_build_failures > 0 {
                    kinds.push(format!("+{} build", commas(s.new_build_failures as u64)));
                }
                if s.new_test_failures > 0 {
                    kinds.push(format!("+{} test", commas(s.new_test_failures as u64)));
                }
                format!("{}: {}", s.label, kinds.join(", "))
            })
            .collect();
        let _ = writeln!(out, "❌ **New failures** — {}.", parts.join("; "));
    }

    for s in stats.iter().filter(|s| !s.available) {
        let _ = writeln!(
            out,
            "❌ **Suite errored** — {} produced no usable report.",
            s.label
        );
    }
    for s in stats.iter().filter(|s| s.classification_failed()) {
        let _ = writeln!(
            out,
            "❌ **Harness error** — {}: benchmark data matched no recognized toolchain naming.",
            s.label
        );
    }
    let unbaselined: Vec<String> = stats
        .iter()
        .filter(|s| s.unbaselined_runs > 0)
        .map(|s| {
            format!(
                "{}: {} runs ({} failures)",
                s.label,
                s.unbaselined_runs,
                commas(s.unbaselined_failures as u64)
            )
        })
        .collect();
    if !unbaselined.is_empty() {
        let _ = writeln!(
            out,
            "⚠️ **No baseline** — {} have no `main` counterpart; their failures are not compared.",
            unbaselined.join("; ")
        );
    }
    let _ = writeln!(out);
}

fn render_results_table(out: &mut String, stats: &[SuiteStats]) {
    let _ = writeln!(out, "| Suite | New failures | Size Δ | Gas Δ | Report |");
    let _ = writeln!(out, "|---|---|---|---|---|");
    for s in stats {
        if !s.available {
            let _ = writeln!(
                out,
                "| {} | ❌ no report — suite errored | — | — | — |",
                s.label
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
fn render_new_failures(out: &mut String, stats: &[SuiteStats]) {
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
fn render_output_changes(out: &mut String, stats: &[SuiteStats]) {
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

fn render_compile_time(out: &mut String, stats: &[SuiteStats]) {
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

fn render_baselines(out: &mut String, stats: &[SuiteStats]) {
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
    use super::*;
    use crate::benchmark::test::Test;
    use crate::benchmark::test::metadata::Metadata;
    use crate::benchmark::test::run::Run;
    use crate::benchmark::test::selector::Selector;

    fn contract_test(project: &str, contract: &str, runs: &[(&str, u64, u64)]) -> (String, Test) {
        let selector = Selector {
            project: project.to_owned(),
            case: Some(contract.to_owned()),
            input: None,
        };
        let mut test = Test::new(Metadata::new(selector.clone(), vec![]));
        for (mode, deploy_size, gas) in runs {
            let mut run = Run::default();
            run.size.push(*deploy_size);
            run.gas.push(*gas);
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    fn failure_test(project: &str, runs: &[(&str, usize, usize)]) -> (String, Test) {
        let selector = Selector {
            project: project.to_owned(),
            case: None,
            input: None,
        };
        let mut test = Test::new(Metadata::new(selector.clone(), vec![]));
        for (mode, build_failures, test_failures) in runs {
            let run = Run {
                build_failures: *build_failures,
                test_failures: *test_failures,
                ..Default::default()
            };
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    fn compile_test(project: &str, runs: &[(&str, u64)]) -> (String, Test) {
        let selector = Selector {
            project: project.to_owned(),
            case: None,
            input: None,
        };
        let mut test = Test::new(Metadata::new(selector.clone(), vec![]));
        for (mode, ms) in runs {
            let mut run = Run::default();
            run.compilation_time.push(*ms);
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    /// Merges the given tests by selector, like the real report ingestion
    /// does — a project's failure and compile-time entries share one key.
    fn suite(label: &str, gas_is_gate: bool, tests: Vec<(String, Test)>) -> SummarySuite {
        let mut benchmark = Benchmark::default();
        for (name, test) in tests {
            let entry = benchmark
                .tests
                .entry(name)
                .or_insert_with(|| Test::new(test.metadata.clone()));
            for (mode, run) in test.runs {
                entry.runs.entry(mode).or_default().extend(&run);
            }
        }
        SummarySuite {
            label: label.to_owned(),
            report_file: format!("{}-report.xlsx", label.to_lowercase()),
            benchmark: Some(benchmark),
            report_url: None,
            gas_is_gate,
        }
    }

    fn unavailable(label: &str) -> SummarySuite {
        SummarySuite {
            label: label.to_owned(),
            report_file: format!("{}-report.xlsx", label.to_lowercase()),
            benchmark: None,
            report_url: None,
            gas_is_gate: false,
        }
    }

    #[test]
    fn output_preserving_when_pr_matches_main() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("00.solx-main-solx-E", 100, 5000),
                ("01.solx-solx-E", 100, 5000),
            ],
        )];
        let out = render(&[suite("solx-tester", true, tests)]);
        assert!(out.contains("✅ **Output-preserving**"), "{out}");
        assert!(out.contains("✅ **No new failures**"), "{out}");
        assert!(!out.contains("largest size changes"), "{out}");
    }

    #[test]
    fn size_diff_reports_output_changed_with_inline_movers() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("00.solx-main-solx-E", 100, 5000),
                ("01.solx-solx-E", 142, 5000),
            ],
        )];
        let out = render(&[suite("solx-tester", true, tests)]);
        assert!(out.contains("⚠️ **Output changed**"), "{out}");
        assert!(out.contains("(+42 B total)"), "{out}");
        assert!(out.contains("largest size changes"), "{out}");
        assert!(
            out.contains("- `C` [EVMLA, deploy] 100 → 142 B (+42.0%)"),
            "{out}"
        );
        // The size change informs; only failures and errored suites alarm.
        assert!(!out.contains("❌"), "{out}");
    }

    #[test]
    fn foundry_gas_jitter_is_reported_with_magnitude_not_gated() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("02.solx-main-legacy", 100, 5000),
                ("03.solx-legacy", 100, 5050),
            ],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("✅ **Output-preserving**"), "{out}");
        assert!(
            out.contains("⚪ jitter 1 of 1, median 1.0% (not gated)"),
            "{out}"
        );
    }

    #[test]
    fn pre_existing_failures_do_not_alarm() {
        let tests = vec![failure_test(
            "proj",
            &[("02.solx-main-legacy", 0, 5), ("03.solx-legacy", 0, 5)],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(
            out.contains(
                "✅ **No new failures** — Foundry's 5 failures already present on `main`."
            ),
            "{out}"
        );
        assert!(out.contains("| ✅ 0 (5 pre-existing) |"), "{out}");
        assert!(!out.contains("❌"), "{out}");
    }

    #[test]
    fn new_failures_alarm_and_are_listed() {
        let tests = vec![failure_test(
            "proj",
            &[("02.solx-main-legacy", 0, 5), ("03.solx-legacy", 1, 7)],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(
            out.contains("❌ **New failures** — Foundry: +1 build, +2 test."),
            "{out}"
        );
        assert!(
            out.contains("| ❌ +1 build, +2 test (5 pre-existing) |"),
            "{out}"
        );
        assert!(
            out.contains("- Foundry: `proj` [legacy] test failures 5 → 7"),
            "{out}"
        );
        assert!(
            out.contains("- Foundry: `proj` [legacy] build failures 0 → 1"),
            "{out}"
        );
    }

    #[test]
    fn compile_time_outlier_is_flagged() {
        let tests = vec![
            compile_test(
                "fast",
                &[("02.solx-main-legacy", 1000), ("03.solx-legacy", 1000)],
            ),
            compile_test(
                "slow",
                &[("02.solx-main-legacy", 1000), ("03.solx-legacy", 1300)],
            ),
        ];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("Project outliers"), "{out}");
        assert!(out.contains("`slow`"), "{out}");
        assert!(out.contains("positive = PR slower"), "{out}");
    }

    #[test]
    fn compile_time_within_noise_is_quiet_and_shows_median() {
        let tests = vec![compile_test(
            "p",
            &[("02.solx-main-legacy", 1000), ("03.solx-legacy", 1010)],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("Within noise"), "{out}");
        assert!(out.contains("+1.0% / +1.0%"), "{out}");
        assert!(!out.contains("Project outliers"), "{out}");
    }

    #[test]
    fn errored_suite_is_flagged_not_dropped() {
        let ok = contract_test(
            "p",
            "C",
            &[
                ("00.solx-main-solx-E", 100, 5000),
                ("01.solx-solx-E", 100, 5000),
            ],
        );
        let out = render(&[suite("solx-tester", true, vec![ok]), unavailable("Foundry")]);
        assert!(
            out.contains("❌ **Suite errored** — Foundry produced no usable report."),
            "{out}"
        );
        assert!(
            out.contains("| Foundry | ❌ no report — suite errored"),
            "{out}"
        );
        // The healthy suite still renders its verdict and row.
        assert!(out.contains("✅ **Output-preserving**"), "{out}");
        assert!(out.contains("| solx-tester |"), "{out}");
    }

    #[test]
    fn unrecognized_toolchain_naming_is_a_loud_error_not_a_green_verdict() {
        // A renamed compiler entry: the main marker still classifies, but the
        // candidate name no longer matches any known pattern.
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("02.mason-main-legacy", 100, 5000),
                ("03.mason-legacy", 100, 5000),
            ],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(
            out.contains(
                "❌ **Harness error** — Foundry: benchmark data matched no recognized toolchain naming."
            ),
            "{out}"
        );
        assert!(
            out.contains("| Foundry | ❌ unrecognized toolchain naming |"),
            "{out}"
        );
        assert!(!out.contains("✅ **Output-preserving**"), "{out}");
    }

    #[test]
    fn unbaselined_runs_are_marked_not_counted_as_new_failures() {
        // The PR run has no main counterpart (e.g. main's run recorded
        // nothing, or the PR enables a new mode).
        let tests = vec![failure_test("proj", &[("03.solx-legacy", 0, 5)])];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(!out.contains("❌ **New failures**"), "{out}");
        assert!(
            out.contains(
                "⚠️ **No baseline** — Foundry: 1 runs (5 failures) have no `main` counterpart"
            ),
            "{out}"
        );
        assert!(out.contains("| ✅ 0, ⚪ 5 unbaselined |"), "{out}");
    }

    #[test]
    fn baselines_compare_common_contracts_only() {
        // C2 is built by the PR but not by solc — it must not skew the
        // comparison as an imaginary zero-size solc contract.
        let tests = vec![
            contract_test(
                "p",
                "C1",
                &[
                    ("00.solc-legacy", 1000, 0),
                    ("02.solx-main-legacy", 1057, 0),
                    ("03.solx-legacy", 1057, 0),
                ],
            ),
            contract_test(
                "p",
                "C2",
                &[
                    ("02.solx-main-legacy", 5000, 0),
                    ("03.solx-legacy", 5000, 0),
                ],
            ),
        ];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("contracts built by both only"), "{out}");
        assert!(out.contains("| Foundry | legacy | +5.7% | — |"), "{out}");
    }

    #[test]
    fn baselines_table_names_baselines_and_direction() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("00.solc-legacy", 1000, 0),
                ("02.solx-main-legacy", 1057, 0),
                ("03.solx-legacy", 1057, 0),
            ],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("full matrix"), "{out}");
        assert!(out.contains("positive = PR larger"), "{out}");
        assert!(
            out.contains("| Suite | Pipeline | vs solc | vs released solx |"),
            "{out}"
        );
        assert!(out.contains("| Foundry | legacy | +5.7% | — |"), "{out}");
        assert!(!out.contains("00.solc"), "{out}");
    }

    #[test]
    fn commas_group_thousands() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(42), "42");
        assert_eq!(commas(47660), "47,660");
        assert_eq!(commas(101098), "101,098");
    }

    /// Compares a rendered comment against its golden fixture — the fixtures
    /// are the reviewable specification of the comment format. Set
    /// `UPDATE_SUMMARY_FIXTURES=1` to regenerate after an intended change.
    fn assert_matches_fixture(name: &str, rendered: &str) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/output/summary/fixtures")
            .join(format!("{name}.md"));
        if std::env::var_os("UPDATE_SUMMARY_FIXTURES").is_some() {
            std::fs::create_dir_all(path.parent().expect("fixture directory"))
                .expect("fixture directory creation");
            std::fs::write(path.as_path(), rendered).expect("fixture writing");
            return;
        }
        let expected = std::fs::read_to_string(path.as_path()).unwrap_or_else(|error| {
            panic!(
                "Fixture {path:?} unreadable ({error}); regenerate with \
                 UPDATE_SUMMARY_FIXTURES=1 cargo test -p solx-benchmark-converter"
            )
        });
        assert_eq!(
            rendered, expected,
            "Rendered summary diverges from fixture {name:?}; if the change is \
             intended, regenerate with UPDATE_SUMMARY_FIXTURES=1 cargo test -p \
             solx-benchmark-converter"
        );
    }

    /// The everyday green run: three suites, pre-existing failures, Foundry
    /// gas jitter, compile time within noise.
    #[test]
    fn fixture_standard_output_preserving() {
        let mut tester = suite(
            "solx-tester",
            true,
            vec![
                contract_test(
                    "solx-solidity",
                    "test/libsolidity/semanticTests/structs/delete_struct.sol",
                    &[
                        ("00.solx-main-solx-E-M3B3-0.8.34", 214, 85_899),
                        ("01.solx-solx-E-M3B3-0.8.34", 214, 85_899),
                        ("00.solx-main-solx-Y-M3B3-0.8.34", 198, 85_412),
                        ("01.solx-solx-Y-M3B3-0.8.34", 198, 85_412),
                    ],
                ),
                contract_test(
                    "tests/solidity",
                    "simple/default.sol",
                    &[
                        ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                        ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                    ],
                ),
            ],
        );
        tester.report_url = Some("https://example.com/artifacts/tester".to_owned());

        let mut foundry = suite(
            "Foundry",
            false,
            vec![
                contract_test(
                    "uniswap-v4",
                    "src/PoolManager.sol:PoolManager",
                    &[
                        ("02.solx-main-legacy", 22_104, 812_004),
                        ("03.solx-legacy", 22_104, 812_650),
                        ("02.solx-main-viaIR", 21_876, 809_112),
                        ("03.solx-viaIR", 21_876, 809_112),
                    ],
                ),
                contract_test(
                    "solady",
                    "src/tokens/ERC20.sol:ERC20",
                    &[
                        ("02.solx-main-legacy", 9_412, 412_338),
                        ("03.solx-legacy", 9_412, 412_780),
                        ("02.solx-main-viaIR", 9_268, 410_006),
                        ("03.solx-viaIR", 9_268, 410_006),
                    ],
                ),
                failure_test(
                    "solady",
                    &[("02.solx-main-legacy", 0, 3), ("03.solx-legacy", 0, 3)],
                ),
                compile_test(
                    "uniswap-v4",
                    &[
                        ("02.solx-main-legacy", 48_210),
                        ("03.solx-legacy", 48_530),
                        ("02.solx-main-viaIR", 61_020),
                        ("03.solx-viaIR", 60_800),
                    ],
                ),
                compile_test(
                    "solady",
                    &[
                        ("02.solx-main-legacy", 96_410),
                        ("03.solx-legacy", 96_150),
                        ("02.solx-main-viaIR", 118_240),
                        ("03.solx-viaIR", 119_030),
                    ],
                ),
            ],
        );
        foundry.report_url = Some("https://example.com/artifacts/foundry".to_owned());

        let mut hardhat = suite(
            "Hardhat",
            false,
            vec![
                failure_test(
                    "ethers-project",
                    &[("02.solx-main-legacy", 0, 2), ("03.solx-legacy", 0, 2)],
                ),
                compile_test(
                    "ethers-project",
                    &[
                        ("02.solx-main-legacy", 15_020),
                        ("03.solx-legacy", 15_110),
                        ("02.solx-main-viaIR", 18_490),
                        ("03.solx-viaIR", 18_530),
                    ],
                ),
            ],
        );
        hardhat.report_url = Some("https://example.com/artifacts/hardhat".to_owned());

        let out = render(&[tester, foundry, hardhat]);
        assert_matches_fixture("standard-output-preserving", &out);
    }

    /// Size and gated-gas differences: the warning verdict, inline movers,
    /// and the "+N more" truncation past `MAX_LISTED`.
    #[test]
    fn fixture_output_changed() {
        let tester = suite(
            "solx-tester",
            true,
            vec![contract_test(
                "solx-solidity",
                "test/libsolidity/semanticTests/structs/delete_struct.sol",
                &[
                    ("00.solx-main-solx-Y-M3B3-0.8.34", 214, 85_899),
                    ("01.solx-solx-Y-M3B3-0.8.34", 214, 85_902),
                ],
            )],
        );

        let mut foundry_tests = Vec::new();
        for index in 0..7u64 {
            foundry_tests.push(contract_test(
                "solady",
                format!("src/C{index}.sol:C{index}").as_str(),
                &[
                    ("02.solx-main-legacy", 1_000 + index * 100, 0),
                    ("03.solx-legacy", 1_000 + index * 100 + 10 + index, 0),
                ],
            ));
        }
        let foundry = suite("Foundry", false, foundry_tests);

        let out = render(&[tester, foundry]);
        assert_matches_fixture("output-changed", &out);
    }

    /// Build and test regressions: the red verdict and the inline listing of
    /// regressed projects.
    #[test]
    fn fixture_new_failures() {
        let foundry = suite(
            "Foundry",
            false,
            vec![
                failure_test(
                    "uniswap-v4",
                    &[("02.solx-main-legacy", 0, 5), ("03.solx-legacy", 1, 7)],
                ),
                failure_test(
                    "solady",
                    &[("02.solx-main-viaIR", 2, 0), ("03.solx-viaIR", 2, 3)],
                ),
                failure_test(
                    "op",
                    &[("02.solx-main-legacy", 0, 4), ("03.solx-legacy", 0, 4)],
                ),
            ],
        );
        let out = render(&[foundry]);
        assert_matches_fixture("new-failures", &out);
    }

    /// Every harness-degradation signal at once: an errored suite, toolchain
    /// naming that matches nothing, and runs without a `main` baseline.
    #[test]
    fn fixture_degraded_harness() {
        let foundry = suite(
            "Foundry",
            false,
            vec![contract_test(
                "p",
                "C",
                &[
                    ("02.mason-main-legacy", 100, 5_000),
                    ("03.mason-legacy", 100, 5_000),
                ],
            )],
        );
        let hardhat = suite(
            "Hardhat",
            false,
            vec![failure_test("hh-project", &[("03.solx-legacy", 0, 5)])],
        );
        let out = render(&[unavailable("solx-tester"), foundry, hardhat]);
        assert_matches_fixture("degraded-harness", &out);
    }

    /// The full-matrix run: solc and released-solx baselines plus a
    /// compile-time project outlier.
    #[test]
    fn fixture_full_matrix() {
        let tester = suite(
            "solx-tester",
            true,
            vec![contract_test(
                "tests/solidity",
                "simple/default.sol",
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                    ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                ],
            )],
        );
        let foundry = suite(
            "Foundry",
            false,
            vec![
                contract_test(
                    "op",
                    "src/L2Bridge.sol:L2Bridge",
                    &[
                        ("00.solc-0.8.34-legacy", 1_000, 0),
                        ("01.solx-latest-legacy", 940, 0),
                        ("02.solx-main-legacy", 902, 0),
                        ("03.solx-legacy", 902, 0),
                        ("00.solc-0.8.34-viaIR", 980, 0),
                        ("01.solx-latest-viaIR", 921, 0),
                        ("02.solx-main-viaIR", 918, 0),
                        ("03.solx-viaIR", 918, 0),
                    ],
                ),
                compile_test(
                    "op",
                    &[("02.solx-main-legacy", 10_000), ("03.solx-legacy", 13_100)],
                ),
                compile_test(
                    "base",
                    &[("02.solx-main-legacy", 20_000), ("03.solx-legacy", 20_150)],
                ),
            ],
        );
        let out = render(&[tester, foundry]);
        assert_matches_fixture("full-matrix", &out);
    }

    /// The smallest possible run: one healthy suite, nothing else collected.
    #[test]
    fn fixture_single_suite() {
        let tester = suite(
            "solx-tester",
            true,
            vec![contract_test(
                "tests/solidity",
                "simple/default.sol",
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                    ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                ],
            )],
        );
        let out = render(&[tester]);
        assert_matches_fixture("single-suite", &out);
    }
}
