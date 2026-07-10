//!
//! Markdown summary of an integration-test benchmark comparison.
//!
//! Renders the one-comment PR summary the integration workflow posts: the
//! correctness gate (bytecode size everywhere + solx-tester gas), new
//! failures, and a threshold-gated compile-time tripwire. The verdict is
//! computed here — the single source of truth shared by every suite — instead
//! of parsing the XLSX back offline.
//!

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::benchmark::Benchmark;

/// A compile-time move on one project large enough to surface individually.
const COMPILE_TIME_PROJECT_THRESHOLD_PERCENT: f64 = 15.0;
/// A suite-aggregate compile-time move large enough to highlight.
const COMPILE_TIME_SUITE_THRESHOLD_PERCENT: f64 = 5.0;
/// Cap on individually-listed items (outliers, top movers) before "+N more".
const MAX_LISTED: usize = 5;

///
/// One suite (solx-tester / Foundry / Hardhat) fed into the summary.
///
pub struct SummarySuite {
    /// Human-readable suite name shown in the table.
    pub label: String,
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
    let gate_failed = stats.iter().any(SuiteStats::gate_failed);

    let mut out = String::new();
    let mode = if full_matrix {
        "full matrix"
    } else {
        "standard"
    };
    let _ = writeln!(out, "### 🧪 Integration tests — {mode} · PR vs `main`\n");

    if gate_failed {
        let _ = writeln!(
            out,
            "❌ **Not output-preserving** — a gated signal changed, a new failure appeared, or a suite produced no report; see below.\n"
        );
    } else {
        let _ = writeln!(
            out,
            "✅ **Output-preserving** — bytecode size identical across all suites, solx-tester gas identical, no new failures.\n"
        );
    }

    render_results_table(&mut out, &stats);
    render_compile_time(&mut out, &stats);
    render_top_movers(&mut out, &stats);
    if full_matrix {
        render_baselines(&mut out, &stats);
    }

    let _ = writeln!(
        out,
        "\n---\n_Gate = **size** (deterministic, all suites) + **solx-tester gas** (deterministic REVM). \
         Foundry/Hardhat gas is fuzz/CREATE-noisy and excluded; compile-time is a non-gating tripwire — \
         authoritative deltas live in the `ci:compile-benchmark` comment._"
    );

    out
}

///
/// Everything the renderer needs about one suite, computed in a single pass.
///
#[derive(Default)]
struct SuiteStats {
    label: String,
    report_url: Option<String>,
    gas_is_gate: bool,
    /// False when the suite was expected but produced no report.
    available: bool,
    project_count: usize,

    size_cells: u64,
    size_diffs: u64,
    size_delta_bytes: i128,
    size_present: bool,

    gas_cells: u64,
    gas_diffs: u64,
    gas_present: bool,

    build_failures: usize,
    test_failures: usize,

    /// Compile-time aggregates keyed by pipeline (legacy / viaIR).
    compile: BTreeMap<String, CompileAggregate>,
    /// Total bytecode size per (role, pipeline) for the full-matrix baselines.
    size_by_role_pipeline: BTreeMap<(Role, String), u64>,
    has_baselines: bool,

    top_size_movers: Vec<Movement>,
    top_gas_movers: Vec<Movement>,
}

#[derive(Default)]
struct CompileAggregate {
    pr_total_ms: u64,
    main_total_ms: u64,
    /// Per-project percentage change, PR vs main.
    per_project: Vec<(String, f64)>,
}

struct Movement {
    label: String,
    mode: String,
    main: u64,
    pr: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    Pr,
    Main,
    Latest,
    Solc,
    Other,
}

impl SuiteStats {
    fn from_suite(suite: &SummarySuite) -> Self {
        let mut stats = SuiteStats {
            label: suite.label.clone(),
            report_url: suite.report_url.clone(),
            gas_is_gate: suite.gas_is_gate,
            available: suite.benchmark.is_some(),
            ..Default::default()
        };
        let Some(benchmark) = suite.benchmark.as_ref() else {
            return stats;
        };

        let mut projects = std::collections::BTreeSet::new();
        for test in benchmark.tests.values() {
            projects.insert(test.metadata.selector.project.clone());
            let contract = test.metadata.selector.case.as_deref().unwrap_or("");

            // Classify every run once and index the PR/main runs by pairing key.
            let mut pr_runs: BTreeMap<String, &crate::benchmark::test::run::Run> = BTreeMap::new();
            let mut main_runs: BTreeMap<String, &crate::benchmark::test::run::Run> =
                BTreeMap::new();
            for (mode, run) in test.runs.iter() {
                let (role, key) = classify(mode);
                match role {
                    Role::Pr => {
                        pr_runs.insert(key, run);
                    }
                    Role::Main => {
                        main_runs.insert(key, run);
                    }
                    Role::Latest | Role::Solc => stats.has_baselines = true,
                    Role::Other => {}
                }

                // Failures are counted on the PR runs only.
                if matches!(role, Role::Pr) {
                    stats.build_failures += run.build_failures;
                    stats.test_failures += run.test_failures;
                }

                // Full-matrix baseline totals: sum bytecode per role and pipeline.
                if matches!(role, Role::Pr | Role::Main | Role::Latest | Role::Solc) {
                    let pipeline = pipeline_of(mode);
                    let bytes = run.average_size() + run.average_runtime_size();
                    if bytes != 0 {
                        *stats
                            .size_by_role_pipeline
                            .entry((role, pipeline))
                            .or_default() += bytes;
                    }
                }

                // Compile-time aggregates (project-level runs carry it).
                if !run.compilation_time.is_empty()
                    && let Role::Pr | Role::Main = role
                {
                    let pipeline = pipeline_of(mode);
                    let entry = stats.compile.entry(pipeline).or_default();
                    let ms = run.average_compilation_time();
                    match role {
                        Role::Pr => entry.pr_total_ms += ms,
                        Role::Main => entry.main_total_ms += ms,
                        _ => {}
                    }
                }
            }

            // Correctness diff: pair each PR run with its main counterpart.
            for (key, pr) in pr_runs.iter() {
                let Some(main) = main_runs.get(key) else {
                    continue;
                };
                let mode = pretty_mode(key);

                for (pr_v, main_v) in [
                    (pr.average_size(), main.average_size()),
                    (pr.average_runtime_size(), main.average_runtime_size()),
                ] {
                    if pr_v == 0 && main_v == 0 {
                        continue;
                    }
                    stats.size_present = true;
                    stats.size_cells += 1;
                    if pr_v != main_v {
                        stats.size_diffs += 1;
                        stats.size_delta_bytes += pr_v as i128 - main_v as i128;
                        push_movement(&mut stats.top_size_movers, contract, &mode, main_v, pr_v);
                    }
                }

                let (pr_gas, main_gas) = (pr.average_gas(), main.average_gas());
                if pr_gas != 0 || main_gas != 0 {
                    stats.gas_present = true;
                    stats.gas_cells += 1;
                    if pr_gas != main_gas {
                        stats.gas_diffs += 1;
                        if suite.gas_is_gate {
                            push_movement(
                                &mut stats.top_gas_movers,
                                contract,
                                &mode,
                                main_gas,
                                pr_gas,
                            );
                        }
                    }
                }
            }
        }

        // Per-project compile-time percentages (for outlier detection) need a
        // second pass now that pipelines are known.
        for test in benchmark.tests.values() {
            let project = test.metadata.selector.project.as_str();
            let mut pr: BTreeMap<String, u64> = BTreeMap::new();
            let mut main: BTreeMap<String, u64> = BTreeMap::new();
            for (mode, run) in test.runs.iter() {
                if run.compilation_time.is_empty() {
                    continue;
                }
                match classify(mode).0 {
                    Role::Pr => {
                        pr.insert(pipeline_of(mode), run.average_compilation_time());
                    }
                    Role::Main => {
                        main.insert(pipeline_of(mode), run.average_compilation_time());
                    }
                    _ => {}
                }
            }
            for (pipeline, &pr_ms) in pr.iter() {
                if let Some(&main_ms) = main.get(pipeline)
                    && main_ms != 0
                {
                    let pct = (pr_ms as f64 - main_ms as f64) / main_ms as f64 * 100.0;
                    stats
                        .compile
                        .entry(pipeline.clone())
                        .or_default()
                        .per_project
                        .push((project.to_owned(), pct));
                }
            }
        }

        stats.project_count = projects.len();
        sort_movers_by_magnitude(&mut stats.top_size_movers);
        sort_movers_by_magnitude(&mut stats.top_gas_movers);
        stats
    }

    fn gate_failed(&self) -> bool {
        !self.available
            || self.size_diffs > 0
            || (self.gas_is_gate && self.gas_diffs > 0)
            || self.build_failures > 0
            || self.test_failures > 0
    }

    fn failures_cell(&self) -> String {
        if self.build_failures == 0 && self.test_failures == 0 {
            "✅ 0".to_owned()
        } else {
            format!(
                "❌ {} build / {} test",
                self.build_failures, self.test_failures
            )
        }
    }

    fn size_cell(&self) -> String {
        if !self.size_present {
            return "—".to_owned();
        }
        if self.size_diffs == 0 {
            format!("✅ `0 / {}`", commas(self.size_cells))
        } else {
            format!(
                "❌ `{} / {}` ({:+} B)",
                commas(self.size_diffs),
                commas(self.size_cells),
                self.size_delta_bytes
            )
        }
    }

    fn gas_cell(&self) -> String {
        if !self.gas_present {
            return "—".to_owned();
        }
        if !self.gas_is_gate {
            return "⚪ noise (excluded)".to_owned();
        }
        if self.gas_diffs == 0 {
            format!("✅ `0 / {}`", commas(self.gas_cells))
        } else {
            format!(
                "❌ `{} / {}`",
                commas(self.gas_diffs),
                commas(self.gas_cells)
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
            Some(url) => format!("[xlsx ↓]({url})"),
            None => "—".to_owned(),
        }
    }
}

fn render_results_table(out: &mut String, stats: &[SuiteStats]) {
    let _ = writeln!(out, "| Suite | Failures | Size Δ | Gas Δ | Report |");
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

fn render_compile_time(out: &mut String, stats: &[SuiteStats]) {
    let with_ct: Vec<&SuiteStats> = stats.iter().filter(|s| !s.compile.is_empty()).collect();
    if with_ct.is_empty() {
        return;
    }

    let pipelines = ["legacy", "viaIR"];
    let _ = writeln!(
        out,
        "\n**Compile time** — wall-clock, shared runner (authoritative Δ in `ci:compile-benchmark`)\n"
    );
    let _ = writeln!(out, "| Suite | legacy | viaIR |");
    let _ = writeln!(out, "|---|---|---|");

    let mut any_suite_flag = false;
    let mut outliers: Vec<(String, String, f64)> = Vec::new();
    for s in &with_ct {
        let mut cells = Vec::new();
        for pipeline in pipelines {
            let cell = match s.compile.get(pipeline) {
                Some(agg) if agg.main_total_ms != 0 => {
                    let pct = (agg.pr_total_ms as f64 - agg.main_total_ms as f64)
                        / agg.main_total_ms as f64
                        * 100.0;
                    if pct.abs() >= COMPILE_TIME_SUITE_THRESHOLD_PERCENT {
                        any_suite_flag = true;
                        format!("⚠️ **{}**", percent(pct))
                    } else {
                        percent(pct)
                    }
                }
                _ => "—".to_owned(),
            };
            cells.push(cell);
            if let Some(agg) = s.compile.get(pipeline) {
                for (project, pct) in agg.per_project.iter() {
                    if pct.abs() >= COMPILE_TIME_PROJECT_THRESHOLD_PERCENT {
                        outliers.push((project.clone(), pipeline.to_owned(), *pct));
                    }
                }
            }
        }
        let _ = writeln!(
            out,
            "| {} ({}) | {} | {} |",
            s.label, s.project_count, cells[0], cells[1]
        );
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

fn render_top_movers(out: &mut String, stats: &[SuiteStats]) {
    let has_movers = stats
        .iter()
        .any(|s| !s.top_size_movers.is_empty() || !s.top_gas_movers.is_empty());
    if !has_movers {
        return;
    }
    let _ = writeln!(
        out,
        "\n<details><summary>Top movers (output changed)</summary>\n"
    );
    for s in stats {
        for (title, movers) in [("size", &s.top_size_movers), ("gas", &s.top_gas_movers)] {
            if movers.is_empty() {
                continue;
            }
            let _ = writeln!(out, "**{} — {title}:**", s.label);
            for m in movers.iter().take(MAX_LISTED) {
                let pct = if m.main != 0 {
                    (m.pr as f64 - m.main as f64) / m.main as f64 * 100.0
                } else {
                    0.0
                };
                let _ = writeln!(
                    out,
                    "- `{}` [{}] {} → {} ({})",
                    m.label,
                    m.mode,
                    commas(m.main),
                    commas(m.pr),
                    percent(pct)
                );
            }
            let extra = movers.len().saturating_sub(MAX_LISTED);
            if extra > 0 {
                let _ = writeln!(out, "- +{extra} more");
            }
        }
    }
    let _ = writeln!(out, "\n</details>");
}

fn render_baselines(out: &mut String, stats: &[SuiteStats]) {
    let relevant: Vec<&SuiteStats> = stats.iter().filter(|s| s.has_baselines).collect();
    if relevant.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n<details><summary>Bytecode size vs solc / released solx (full matrix)</summary>\n"
    );
    let _ = writeln!(
        out,
        "| Suite | pipeline | vs `00.solc` | vs `01.solx` (latest) |"
    );
    let _ = writeln!(out, "|---|---|---|---|");
    for s in relevant {
        for pipeline in ["legacy", "viaIR"] {
            let pr = s
                .size_by_role_pipeline
                .get(&(Role::Pr, pipeline.to_owned()))
                .copied();
            let Some(pr) = pr else { continue };
            let vs = |role: Role| -> String {
                match s.size_by_role_pipeline.get(&(role, pipeline.to_owned())) {
                    Some(&base) if base != 0 => {
                        percent((pr as f64 - base as f64) / base as f64 * 100.0)
                    }
                    _ => "—".to_owned(),
                }
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                s.label,
                pipeline,
                vs(Role::Solc),
                vs(Role::Latest)
            );
        }
    }
    let _ = writeln!(out, "\n</details>");
}

///
/// Classifies a run's mode string into a role and its pairing key.
///
/// Mode strings look like `02.solx-main-legacy` or `01.solx-solx-E-M3B3-0.8.34`.
/// The pairing key is every token after the leading `NN.solx`/`NN.solc`
/// identifier, minus the role markers `main`/`latest`, so a PR run pairs with
/// its main counterpart.
///
fn classify(mode: &str) -> (Role, String) {
    let mut tokens = mode.split('-');
    let head = tokens.next().unwrap_or("");
    let rest: Vec<&str> = tokens.collect();

    let role = if head.ends_with(".solc") {
        Role::Solc
    } else if rest.contains(&"latest") {
        Role::Latest
    } else if rest.contains(&"main") {
        Role::Main
    } else if head.ends_with(".solx") {
        Role::Pr
    } else {
        Role::Other
    };

    let key = rest
        .into_iter()
        .filter(|t| *t != "main" && *t != "latest")
        .collect::<Vec<_>>()
        .join("-");
    (role, key)
}

/// The compilation pipeline (`legacy` / `viaIR`) a mode belongs to, or its
/// trailing token otherwise.
fn pipeline_of(mode: &str) -> String {
    mode.rsplit('-').next().unwrap_or("").to_owned()
}

/// A pairing key back to a compact human-readable mode (drop the leading solx
/// token that the key already stripped).
fn pretty_mode(key: &str) -> String {
    key.replace('-', "/")
}

fn push_movement(movers: &mut Vec<Movement>, label: &str, mode: &str, main: u64, pr: u64) {
    movers.push(Movement {
        label: label.to_owned(),
        mode: mode.to_owned(),
        main,
        pr,
    });
}

/// Orders movers by descending magnitude so the renderer lists the biggest
/// first and counts the rest as "+N more".
fn sort_movers_by_magnitude(movers: &mut [Movement]) {
    movers.sort_by(|a, b| {
        let da = (a.pr as i128 - a.main as i128).unsigned_abs();
        let db = (b.pr as i128 - b.main as i128).unsigned_abs();
        db.cmp(&da)
    });
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

    fn suite(label: &str, gas_is_gate: bool, tests: Vec<(String, Test)>) -> SummarySuite {
        let mut benchmark = Benchmark::default();
        benchmark.tests.extend(tests);
        SummarySuite {
            label: label.to_owned(),
            benchmark: Some(benchmark),
            report_url: None,
            gas_is_gate,
        }
    }

    fn unavailable(label: &str) -> SummarySuite {
        SummarySuite {
            label: label.to_owned(),
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
        assert!(!out.contains("Top movers"), "{out}");
    }

    #[test]
    fn size_diff_trips_the_gate_and_lists_movers() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("00.solx-main-solx-E", 100, 5000),
                ("01.solx-solx-E", 142, 5000),
            ],
        )];
        let out = render(&[suite("solx-tester", true, tests)]);
        assert!(out.contains("❌ **Not output-preserving**"), "{out}");
        assert!(out.contains("(+42 B)"), "{out}");
        assert!(out.contains("Top movers"), "{out}");
    }

    #[test]
    fn foundry_gas_jitter_does_not_trip_the_gate() {
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
        assert!(out.contains("⚪ noise (excluded)"), "{out}");
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
    }

    #[test]
    fn compile_time_within_noise_is_quiet() {
        let tests = vec![compile_test(
            "p",
            &[("02.solx-main-legacy", 1000), ("03.solx-legacy", 1010)],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("Within noise"), "{out}");
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
        assert!(out.contains("❌ **Not output-preserving**"), "{out}");
        assert!(
            out.contains("| Foundry | ❌ no report — suite errored"),
            "{out}"
        );
        // The healthy suite still renders its row.
        assert!(out.contains("| solx-tester |"), "{out}");
    }

    #[test]
    fn commas_group_thousands() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(42), "42");
        assert_eq!(commas(47660), "47,660");
        assert_eq!(commas(101098), "101,098");
    }
}
