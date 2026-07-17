//!
//! Everything the renderer needs about one suite, computed in a single pass.
//!
//! One pass over a suite's benchmark pairs every PR run with its `main`
//! counterpart and reduces the pairs to numbers. Nothing here produces
//! markdown — how the numbers read is the rendering layer's decision.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::output::summary::SuiteOutcome;
use crate::output::summary::SummarySuite;
use crate::output::summary::compile_aggregate::CompileAggregate;
use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::failure_regression::FailureRegression;
use crate::output::summary::failure_regressions::FailureRegressions;
use crate::output::summary::paired_bytes::PairedBytes;
use crate::output::summary::suite_failures::SuiteFailures;
use crate::output::summary::suite_row::SuiteRow;
use crate::output::summary::toolchain_matrix::ToolchainMatrix;
use crate::output::summary::top_movers::TopMovers;
use crate::role::Role;
use crate::utils::commas;
use crate::utils::median;
use crate::utils::relative_percent;

///
/// Everything the renderer needs about one suite, computed in a single pass.
///
#[derive(Default)]
pub struct SuiteStats {
    pub label: String,
    pub report_file: String,
    pub report_url: Option<String>,
    pub gas_is_gate: bool,
    /// False when the suite was expected but produced no report.
    pub available: bool,
    /// How the suite's workflow step ended.
    pub outcome: SuiteOutcome,
    pub project_count: usize,
    /// Total runs seen, and how many classified as the PR toolchain — data
    /// with zero PR runs means the toolchain naming drifted from classify().
    pub total_runs: usize,
    pub pr_runs_seen: usize,
    /// PR runs that found a `main` counterpart — the failure verdict only
    /// means something when at least one suite compared something.
    pub paired_runs: usize,
    /// PR runs with no main counterpart, and the failures recorded on them.
    /// They have nothing to compare against, so they are surfaced as
    /// unbaselined rather than counted as regressions against zero.
    pub unbaselined_runs: usize,
    pub unbaselined_failures: usize,
    /// Main runs with no PR counterpart — a comparison set that silently
    /// shrank (a crash or skip on the PR side) must be surfaced, not
    /// dropped by the PR-keyed pairing.
    pub main_orphan_runs: usize,
    pub main_orphan_failures: usize,
    /// Mode strings matching no declared toolchain name — always surfaced as
    /// a harness error, whether or not PR runs are present.
    pub unrecognized_modes: BTreeSet<String>,
    /// Recognized runs whose mode carries no recognized pipeline token —
    /// their per-pipeline data is excluded and the drift surfaced loudly.
    pub unrecognized_pipelines: BTreeSet<String>,

    pub size: DiffCounter,
    /// Size pairs the PR emitted and `main` did not: no baseline exists, so
    /// they are excluded from the diff count and stated apart in the cell.
    /// The mirror — `main` emitted bytecode the PR lost — is a regression, and
    /// counts as a differing pair rather than landing here.
    pub size_one_sided: u64,
    pub gas: DiffCounter,
    /// Relative gas differences seen on a non-gating suite, in percent. The
    /// median is reported — a max would routinely be a huge but meaningless
    /// CREATE-deploy outlier.
    pub gas_jitter_percents: Vec<f64>,
    /// Non-gated differing pairs only one side measured, in either direction:
    /// the percentage between a measurement and its absence is meaningless, so
    /// they are reported next to the jitter median rather than folded into it
    /// as a fabricated sample.
    pub gas_one_sided: u64,

    /// Failures on the PR runs in excess of their main counterparts.
    pub new_build_failures: usize,
    pub new_test_failures: usize,
    /// Failures already present on the paired main runs — a failing run that
    /// vanished from the PR side reports as main-only, never as pre-existing.
    pub baseline_build_failures: usize,
    pub baseline_test_failures: usize,
    /// The rows behind `new_*_failures`, for the inline listing.
    pub failure_regressions: FailureRegressions,

    /// Compile-time aggregates keyed by pipeline (legacy / viaIR).
    pub compile: BTreeMap<String, CompileAggregate>,
    /// PR and baseline bytecode totals per (baseline role, pipeline), summed
    /// only over contracts both toolchains emitted — a toolchain that failed
    /// some builds is excluded from the comparison, not counted as 0.
    pub baseline_pairs: BTreeMap<(Role, String), PairedBytes>,
    pub has_baselines: bool,

    pub top_size_movers: TopMovers,
    pub top_gas_movers: TopMovers,
}

impl SuiteStats {
    pub fn from_suite(suite: &SummarySuite) -> Self {
        let mut stats = SuiteStats {
            label: suite.kind.label().to_owned(),
            report_file: suite.kind.report_file().to_owned(),
            report_url: suite.report_url.clone(),
            gas_is_gate: suite.kind.gas_is_gate(),
            available: suite.benchmark.is_some(),
            outcome: suite.outcome,
            ..Default::default()
        };
        let Some(benchmark) = suite.benchmark.as_ref() else {
            return stats;
        };

        let mut projects = BTreeSet::new();
        for test in benchmark.tests.values() {
            let selector = &test.metadata.selector;
            projects.insert(selector.project.clone());
            let row = selector
                .case
                .as_deref()
                .unwrap_or(selector.project.as_str());
            let row_label = match selector.input.as_ref() {
                Some(input) if !input.is_deploy() => format!("{row}[{input}]"),
                _ => row.to_owned(),
            };

            let mut pr_runs: BTreeMap<String, &crate::benchmark::test::run::Run> = BTreeMap::new();
            let mut main_runs: BTreeMap<String, &crate::benchmark::test::run::Run> =
                BTreeMap::new();
            let mut by_role_pipeline: BTreeMap<(Role, String), u64> = BTreeMap::new();
            let mut pr_compile: BTreeMap<String, u64> = BTreeMap::new();
            let mut main_compile: BTreeMap<String, u64> = BTreeMap::new();
            for (mode, run) in test.runs.iter() {
                stats.total_runs += 1;
                let (role, key) = suite.kind.matrix().classify(mode);
                match role {
                    Role::Pr => {
                        stats.pr_runs_seen += 1;
                        pr_runs.insert(key, run);
                    }
                    Role::Main => {
                        main_runs.insert(key, run);
                    }
                    Role::Latest | Role::Solc => stats.has_baselines = true,
                    Role::Other => {
                        stats.unrecognized_modes.insert(mode.clone());
                    }
                }

                let pipeline = match ToolchainMatrix::pipeline_of(mode) {
                    Some(pipeline) => pipeline,
                    None => {
                        if role != Role::Other {
                            stats.unrecognized_pipelines.insert(mode.clone());
                        }
                        continue;
                    }
                };

                if matches!(role, Role::Pr | Role::Main | Role::Latest | Role::Solc) {
                    let bytes = run.average_size() + run.average_runtime_size();
                    if bytes != 0 {
                        *by_role_pipeline
                            .entry((role, pipeline.clone()))
                            .or_default() += bytes;
                    }
                }

                if !run.compilation_time.is_empty() {
                    let ms = run.average_compilation_time();
                    match role {
                        Role::Pr => *pr_compile.entry(pipeline).or_default() += ms,
                        Role::Main => *main_compile.entry(pipeline).or_default() += ms,
                        _ => {}
                    }
                }
            }

            for pipeline in pr_compile.keys().chain(main_compile.keys()) {
                stats.compile.entry(pipeline.clone()).or_default();
            }
            for (pipeline, &pr_ms) in pr_compile.iter() {
                let Some(&main_ms) = main_compile.get(pipeline) else {
                    continue;
                };
                let entry = stats.compile.entry(pipeline.clone()).or_default();
                entry.pr_total_ms += pr_ms;
                entry.main_total_ms += main_ms;
                if let Some(pct) = relative_percent(pr_ms, main_ms) {
                    entry
                        .per_project
                        .push((test.metadata.selector.project.clone(), pct));
                }
            }

            for ((role, pipeline), &base_bytes) in by_role_pipeline.iter() {
                if !matches!(role, Role::Solc | Role::Latest) {
                    continue;
                }
                if let Some(&pr_bytes) = by_role_pipeline.get(&(Role::Pr, pipeline.clone())) {
                    let entry = stats
                        .baseline_pairs
                        .entry((*role, pipeline.clone()))
                        .or_default();
                    entry.pr += pr_bytes;
                    entry.baseline += base_bytes;
                }
            }

            for (key, pr) in pr_runs.iter() {
                let Some(main) = main_runs.get(key) else {
                    stats.unbaselined_runs += 1;
                    stats.unbaselined_failures += pr.failures_count();
                    continue;
                };
                stats.paired_runs += 1;
                stats.baseline_build_failures += main.build_failures_count().unwrap_or_default();
                stats.baseline_test_failures += main.test_failures_count().unwrap_or_default();
                let mode = ToolchainMatrix::humanize_mode(key);

                for (is_build, main_v, pr_v) in [
                    (true, main.build_failures_count(), pr.build_failures_count()),
                    (false, main.test_failures_count(), pr.test_failures_count()),
                ] {
                    let (Some(main_v), Some(pr_v)) = (main_v, pr_v) else {
                        continue;
                    };
                    if pr_v > main_v {
                        let counter = if is_build {
                            &mut stats.new_build_failures
                        } else {
                            &mut stats.new_test_failures
                        };
                        *counter += pr_v - main_v;
                        stats.failure_regressions.push(FailureRegression {
                            label: row_label.clone(),
                            mode: mode.clone(),
                            kind: if is_build { "build" } else { "test" },
                            main: main_v,
                            pr: pr_v,
                        });
                    }
                }

                for (kind, pr_v, main_v) in [
                    ("deploy", pr.average_size(), main.average_size()),
                    (
                        "runtime",
                        pr.average_runtime_size(),
                        main.average_runtime_size(),
                    ),
                ] {
                    if main_v == 0 && pr_v != 0 {
                        stats.size_one_sided += 1;
                    } else if stats.size.observe(pr_v, main_v) {
                        stats.top_size_movers.push(
                            row_label.as_str(),
                            format!("{mode}, {kind}").as_str(),
                            main_v,
                            pr_v,
                        );
                    }
                }

                let (pr_gas, main_gas) = (pr.average_gas(), main.average_gas());
                if stats.gas.observe(pr_gas, main_gas) {
                    if suite.kind.gas_is_gate() {
                        stats.top_gas_movers.push(
                            row_label.as_str(),
                            mode.as_str(),
                            main_gas,
                            pr_gas,
                        );
                    } else if (pr_gas == 0) != (main_gas == 0) {
                        stats.gas_one_sided += 1;
                    } else if let Some(pct) = relative_percent(pr_gas, main_gas) {
                        stats.gas_jitter_percents.push(pct.abs());
                    }
                }
            }

            for (key, main) in main_runs.iter() {
                if !pr_runs.contains_key(key) {
                    stats.main_orphan_runs += 1;
                    stats.main_orphan_failures += main.failures_count();
                }
            }
        }

        stats.project_count = projects.len();
        stats
    }

    pub fn new_failures(&self) -> usize {
        self.new_build_failures + self.new_test_failures
    }

    pub fn baseline_failures(&self) -> usize {
        self.baseline_build_failures + self.baseline_test_failures
    }

    /// The benchmark had runs but none classified as the PR toolchain — the
    /// naming convention drifted from the declared tables; better a loud
    /// error than a green comment over empty data.
    pub fn classification_failed(&self) -> bool {
        self.available && self.total_runs > 0 && self.pr_runs_seen == 0
    }

    /// The report parsed but contains no runs at all — a suite that tested
    /// nothing must not render as a clean pass.
    pub fn is_empty_report(&self) -> bool {
        self.available && self.total_runs == 0
    }

    /// An available suite carrying only the given label, for the verdict tests.
    #[cfg(test)]
    pub fn available(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            available: true,
            ..Default::default()
        }
    }

    /// Jitter medians below this render as "<0.1%": the floor under which the
    /// one-decimal display precision would round to a bare 0.0%.
    const JITTER_MEDIAN_FLOOR_PERCENT: f64 = 0.05;

    /// The suite's row in the results table. A suite with no comparable data
    /// dashes its measurement columns rather than rendering a zero.
    pub fn row(&self) -> SuiteRow {
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
                SuiteFailures::kinds(self.new_build_failures, self.new_test_failures)
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
                    Some(med) if med >= Self::JITTER_MEDIAN_FLOOR_PERCENT => format!("{med:.1}%"),
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

    pub fn suite_cell(&self) -> String {
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
