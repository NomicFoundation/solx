//!
//! Markdown summary of an integration-test benchmark comparison.
//!
//! Renders the one-comment PR summary the integration workflow posts: the
//! correctness verdict (bytecode size everywhere + solx-tester gas), new
//! failures vs main, and a threshold-gated compile-time tripwire. The verdict
//! is computed here — the single source of truth shared by every suite —
//! instead of parsing the XLSX back offline.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write;

use crate::benchmark::Benchmark;

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
    /// Relative gas differences seen on a non-gating suite, in percent. The
    /// median is reported — a max would routinely be a huge but meaningless
    /// CREATE-deploy outlier.
    gas_jitter_percents: Vec<f64>,

    /// Failures on the PR runs in excess of their main counterparts.
    new_build_failures: usize,
    new_test_failures: usize,
    /// Failures already present on the main runs.
    baseline_build_failures: usize,
    baseline_test_failures: usize,
    /// The rows behind `new_*_failures`, for the inline listing.
    failure_regressions: Vec<FailureRegression>,

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

struct FailureRegression {
    label: String,
    mode: String,
    kind: &'static str,
    main: usize,
    pr: usize,
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
            let row_label = if contract.is_empty() {
                test.metadata.selector.project.as_str()
            } else {
                contract
            };

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
                        stats.baseline_build_failures += run.build_failures;
                        stats.baseline_test_failures += run.test_failures;
                        main_runs.insert(key, run);
                    }
                    Role::Latest | Role::Solc => stats.has_baselines = true,
                    Role::Other => {}
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

            // Pair each PR run with its main counterpart. Failures compare even
            // without a counterpart (baseline 0: everything is new); size/gas
            // need both sides.
            for (key, pr) in pr_runs.iter() {
                let main = main_runs.get(key);
                let mode = humanize_mode(key);

                let (main_build, main_test) =
                    main.map_or((0, 0), |m| (m.build_failures, m.test_failures));
                for (kind, main_v, pr_v) in [
                    ("build", main_build, pr.build_failures),
                    ("test", main_test, pr.test_failures),
                ] {
                    if pr_v > main_v {
                        match kind {
                            "build" => stats.new_build_failures += pr_v - main_v,
                            _ => stats.new_test_failures += pr_v - main_v,
                        }
                        stats.failure_regressions.push(FailureRegression {
                            label: row_label.to_owned(),
                            mode: mode.clone(),
                            kind,
                            main: main_v,
                            pr: pr_v,
                        });
                    }
                }

                let Some(main) = main else {
                    continue;
                };

                for (kind, pr_v, main_v) in [
                    ("deploy", pr.average_size(), main.average_size()),
                    (
                        "runtime",
                        pr.average_runtime_size(),
                        main.average_runtime_size(),
                    ),
                ] {
                    if pr_v == 0 && main_v == 0 {
                        continue;
                    }
                    stats.size_present = true;
                    stats.size_cells += 1;
                    if pr_v != main_v {
                        stats.size_diffs += 1;
                        stats.size_delta_bytes += pr_v as i128 - main_v as i128;
                        push_movement(
                            &mut stats.top_size_movers,
                            row_label,
                            &format!("{mode}, {kind}"),
                            main_v,
                            pr_v,
                        );
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
                                row_label,
                                &mode,
                                main_gas,
                                pr_gas,
                            );
                        } else if main_gas != 0 {
                            let jitter =
                                (pr_gas as f64 - main_gas as f64).abs() / main_gas as f64 * 100.0;
                            stats.gas_jitter_percents.push(jitter);
                        }
                    }
                }
            }
        }

        // Per-project compile-time percentages (for outlier detection and the
        // median) need a second pass now that pipelines are known.
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

    fn new_failures(&self) -> usize {
        self.new_build_failures + self.new_test_failures
    }

    fn baseline_failures(&self) -> usize {
        self.baseline_build_failures + self.baseline_test_failures
    }

    fn failures_cell(&self) -> String {
        let pre = match self.baseline_failures() {
            0 => String::new(),
            n => format!(" ({} pre-existing)", commas(n as u64)),
        };
        if self.new_failures() == 0 {
            format!("✅ 0{pre}")
        } else {
            let mut kinds = Vec::new();
            if self.new_build_failures > 0 {
                kinds.push(format!("+{} build", commas(self.new_build_failures as u64)));
            }
            if self.new_test_failures > 0 {
                kinds.push(format!("+{} test", commas(self.new_test_failures as u64)));
            }
            format!("❌ {}{pre}", kinds.join(", "))
        }
    }

    fn size_cell(&self) -> String {
        if !self.size_present {
            return "⚪ not collected".to_owned();
        }
        if self.size_diffs == 0 {
            format!("✅ 0 of {}", commas(self.size_cells))
        } else {
            format!(
                "⚠️ {} of {} ({:+} B)",
                commas(self.size_diffs),
                commas(self.size_cells),
                self.size_delta_bytes
            )
        }
    }

    fn gas_cell(&self) -> String {
        if !self.gas_present {
            return "⚪ not collected".to_owned();
        }
        if !self.gas_is_gate {
            if self.gas_diffs == 0 {
                return "⚪ no jitter (not gated)".to_owned();
            }
            let med = match median(&self.gas_jitter_percents) {
                Some(med) if med >= 0.05 => format!("{med:.1}%"),
                _ => "<0.1%".to_owned(),
            };
            return format!(
                "⚪ jitter {} of {}, median {med} (not gated)",
                commas(self.gas_diffs),
                commas(self.gas_cells)
            );
        }
        if self.gas_diffs == 0 {
            format!("✅ 0 of {}", commas(self.gas_cells))
        } else {
            format!(
                "⚠️ {} of {}",
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

    /// The report file name shown as link text and referenced by "+N more" lines.
    fn report_file(&self) -> String {
        format!("{}.xlsx", self.label.to_lowercase())
    }

    fn report_cell(&self) -> String {
        match self.report_url.as_deref() {
            Some(url) => format!("[{} ↓]({url})", self.report_file()),
            None => "—".to_owned(),
        }
    }
}

///
/// The verdict lines: output invariance, new failures, and harness health —
/// three independent signals, each stated with its numbers.
///
fn render_verdict(out: &mut String, stats: &[SuiteStats]) {
    let size_cells: u64 = stats.iter().map(|s| s.size_cells).sum();
    let size_diffs: u64 = stats.iter().map(|s| s.size_diffs).sum();
    let size_delta: i128 = stats.iter().map(|s| s.size_delta_bytes).sum();
    let gated: Vec<&SuiteStats> = stats.iter().filter(|s| s.gas_is_gate).collect();
    let gated_gas_cells: u64 = gated.iter().map(|s| s.gas_cells).sum();
    let gated_gas_diffs: u64 = gated.iter().map(|s| s.gas_diffs).sum();
    let gas_label = gated
        .iter()
        .filter(|s| s.gas_present)
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
            clauses.push("no size or gated-gas data was collected".to_owned());
        }
        let _ = writeln!(out, "✅ **Output-preserving** — {}.", clauses.join(", "));
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
            "❌ **Suite errored** — {} produced no report.",
            s.label
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
            let _ = writeln!(out, "- +{extra} more — see {}", s.report_file());
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
            for m in movers.iter().take(MAX_LISTED) {
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
                let _ = writeln!(out, "- +{extra} more — full list in {}", s.report_file());
            }
        }
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
        "\n**Compile time** — wall-clock tripwire, positive = PR slower (authoritative Δ in `ci:compile-benchmark`)\n"
    );
    let _ = writeln!(
        out,
        "| Suite | legacy (agg / median) | viaIR (agg / median) |"
    );
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
            cells.push(cell);
            if let Some(agg) = s.compile.get(pipeline) {
                for (project, pct) in agg.per_project.iter() {
                    if pct.abs() >= COMPILE_TIME_PROJECT_THRESHOLD_PERCENT {
                        outliers.push((project.clone(), pipeline.to_owned(), *pct));
                    }
                }
            }
        }
        let _ = writeln!(out, "| {} | {} | {} |", s.suite_cell(), cells[0], cells[1]);
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
    let relevant: Vec<&SuiteStats> = stats.iter().filter(|s| s.has_baselines).collect();
    if relevant.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n**Bytecode size — PR vs baselines** (positive = PR larger)\n"
    );
    let _ = writeln!(out, "| Suite | Pipeline | vs solc | vs released solx |");
    let _ = writeln!(out, "|---|---|---|---|");
    for s in relevant {
        let pipelines: BTreeSet<&String> = s
            .size_by_role_pipeline
            .keys()
            .filter(|(role, _)| *role == Role::Pr)
            .map(|(_, pipeline)| pipeline)
            .collect();
        for pipeline in pipelines {
            let pr = s
                .size_by_role_pipeline
                .get(&(Role::Pr, pipeline.clone()))
                .copied();
            let Some(pr) = pr else { continue };
            let vs = |role: Role| -> String {
                match s.size_by_role_pipeline.get(&(role, pipeline.clone())) {
                    Some(&base) if base != 0 => {
                        percent((pr as f64 - base as f64) / base as f64 * 100.0)
                    }
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

///
/// A pairing key rendered for humans: the redundant `solx` token dropped and
/// the codegen shorthands spelled out (`E` → EVMLA, `Y` → Yul).
///
fn humanize_mode(key: &str) -> String {
    let tokens: Vec<&str> = key
        .split('-')
        .filter(|token| *token != "solx" && !token.is_empty())
        .map(|token| match token {
            "E" => "EVMLA",
            "Y" => "Yul",
            other => other,
        })
        .collect();
    if tokens.is_empty() {
        key.to_owned()
    } else {
        tokens.join(" ")
    }
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

/// The median of the given percentages, if any were collected.
fn median(pcts: &[f64]) -> Option<f64> {
    if pcts.is_empty() {
        return None;
    }
    let mut pcts = pcts.to_vec();
    pcts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(pcts[pcts.len() / 2])
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
            out.contains("❌ **Suite errored** — Foundry produced no report."),
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
    fn report_link_is_named_after_the_suite() {
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("02.solx-main-legacy", 100, 5000),
                ("03.solx-legacy", 100, 5000),
            ],
        )];
        let mut s = suite("Foundry", false, tests);
        s.report_url = Some("https://example.com/artifact".to_owned());
        let out = render(&[s]);
        assert!(
            out.contains("[foundry.xlsx ↓](https://example.com/artifact)"),
            "{out}"
        );
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
}
