//!
//! Per-suite statistics behind the integration summary.
//!
//! One pass over a suite's benchmark pairs every PR run with its `main`
//! counterpart and reduces the pairs to numbers. Nothing here produces
//! markdown — how the numbers read is the rendering layer's decision.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::SummarySuite;
use super::toolchain::Role;
use super::toolchain::classify;
use super::toolchain::humanize_mode;
use super::toolchain::pipeline_of;

///
/// Counts PR-vs-main comparison pairs and the differing subset.
///
#[derive(Default)]
pub(crate) struct DiffCounter {
    /// Pairs where at least one side produced a value.
    pub(crate) cells: u64,
    /// Pairs whose sides differ.
    pub(crate) diffs: u64,
    /// Signed PR-minus-main total over the differing pairs.
    pub(crate) delta: i128,
}

impl DiffCounter {
    ///
    /// Records one pair, ignoring pairs where neither side produced a value.
    /// Returns whether the recorded pair differs.
    ///
    pub(crate) fn observe(&mut self, pr: u64, main: u64) -> bool {
        if pr == 0 && main == 0 {
            return false;
        }
        self.cells += 1;
        if pr == main {
            return false;
        }
        self.diffs += 1;
        self.delta += pr as i128 - main as i128;
        true
    }

    /// Whether any pair was recorded — false renders as "not collected".
    pub(crate) fn collected(&self) -> bool {
        self.cells > 0
    }
}

///
/// One row's movement between the main and PR toolchains.
///
pub(crate) struct Movement {
    pub(crate) label: String,
    pub(crate) mode: String,
    pub(crate) main: u64,
    pub(crate) pr: u64,
}

///
/// Movements collected for the inline "largest changes" listings.
///
#[derive(Default)]
pub(crate) struct TopMovers(Vec<Movement>);

impl TopMovers {
    pub(crate) fn push(&mut self, label: &str, mode: &str, main: u64, pr: u64) {
        self.0.push(Movement {
            label: label.to_owned(),
            mode: mode.to_owned(),
            main,
            pr,
        });
    }

    /// The movements ordered by descending magnitude, so the renderer lists
    /// the biggest first and counts the rest as "+N more".
    pub(crate) fn ranked(&self) -> Vec<&Movement> {
        let mut movers: Vec<&Movement> = self.0.iter().collect();
        movers.sort_by_key(|movement| {
            std::cmp::Reverse((movement.pr as i128 - movement.main as i128).unsigned_abs())
        });
        movers
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

///
/// Bytecode totals summed over contracts that both the PR and the baseline
/// toolchain emitted.
///
#[derive(Default)]
pub(crate) struct PairedBytes {
    pub(crate) pr: u64,
    pub(crate) baseline: u64,
}

///
/// Compile-time totals for one pipeline.
///
#[derive(Default)]
pub(crate) struct CompileAggregate {
    pub(crate) pr_total_ms: u64,
    pub(crate) main_total_ms: u64,
    /// Per-project percentage change, PR vs main.
    pub(crate) per_project: Vec<(String, f64)>,
}

///
/// One row whose PR run failed more than its main counterpart.
///
pub(crate) struct FailureRegression {
    pub(crate) label: String,
    pub(crate) mode: String,
    pub(crate) kind: &'static str,
    pub(crate) main: usize,
    pub(crate) pr: usize,
}

///
/// Everything the renderer needs about one suite, computed in a single pass.
///
#[derive(Default)]
pub(crate) struct SuiteStats {
    pub(crate) label: String,
    pub(crate) report_file: String,
    pub(crate) report_url: Option<String>,
    pub(crate) gas_is_gate: bool,
    /// False when the suite was expected but produced no report.
    pub(crate) available: bool,
    pub(crate) project_count: usize,
    /// Total runs seen, and how many classified as the PR toolchain — data
    /// with zero PR runs means the toolchain naming drifted from classify().
    pub(crate) total_runs: usize,
    pub(crate) pr_runs_seen: usize,
    /// PR runs that found a `main` counterpart — the failure verdict only
    /// means something when at least one suite compared something.
    pub(crate) paired_runs: usize,
    /// PR runs with no main counterpart, and the failures recorded on them.
    /// They have nothing to compare against, so they are surfaced as
    /// unbaselined rather than counted as regressions against zero.
    pub(crate) unbaselined_runs: usize,
    pub(crate) unbaselined_failures: usize,
    /// Main runs with no PR counterpart — a comparison set that silently
    /// shrank (a crash or skip on the PR side) must be surfaced, not
    /// dropped by the PR-keyed pairing.
    pub(crate) main_orphan_runs: usize,
    pub(crate) main_orphan_failures: usize,
    /// Mode strings matching no declared toolchain name — always surfaced as
    /// a harness error, whether or not PR runs are present.
    pub(crate) unrecognized_modes: BTreeSet<String>,

    pub(crate) size: DiffCounter,
    /// Size pairs where exactly one side produced a value — excluded from
    /// the diff count (nothing to compare) and stated apart in the cell.
    pub(crate) size_one_sided: u64,
    pub(crate) gas: DiffCounter,
    /// Relative gas differences seen on a non-gating suite, in percent. The
    /// median is reported — a max would routinely be a huge but meaningless
    /// CREATE-deploy outlier.
    pub(crate) gas_jitter_percents: Vec<f64>,
    /// Non-gated differing pairs whose `main` side recorded no gas: no
    /// percentage exists, so they are reported next to the jitter median
    /// rather than silently understated by it.
    pub(crate) gas_diffs_without_main: u64,

    /// Failures on the PR runs in excess of their main counterparts.
    pub(crate) new_build_failures: usize,
    pub(crate) new_test_failures: usize,
    /// Failures already present on the main runs.
    pub(crate) baseline_build_failures: usize,
    pub(crate) baseline_test_failures: usize,
    /// The rows behind `new_*_failures`, for the inline listing.
    pub(crate) failure_regressions: Vec<FailureRegression>,

    /// Compile-time aggregates keyed by pipeline (legacy / viaIR).
    pub(crate) compile: BTreeMap<String, CompileAggregate>,
    /// PR and baseline bytecode totals per (baseline role, pipeline), summed
    /// only over contracts both toolchains emitted — a toolchain that failed
    /// some builds is excluded from the comparison, not counted as 0.
    pub(crate) baseline_pairs: BTreeMap<(Role, String), PairedBytes>,
    pub(crate) has_baselines: bool,

    pub(crate) top_size_movers: TopMovers,
    pub(crate) top_gas_movers: TopMovers,
}

impl SuiteStats {
    pub(crate) fn from_suite(suite: &SummarySuite) -> Self {
        let mut stats = SuiteStats {
            label: suite.label.clone(),
            report_file: suite.report_file.clone(),
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

            let mut pr_runs: BTreeMap<String, &crate::benchmark::test::run::Run> = BTreeMap::new();
            let mut main_runs: BTreeMap<String, &crate::benchmark::test::run::Run> =
                BTreeMap::new();
            let mut by_role_pipeline: BTreeMap<(Role, String), u64> = BTreeMap::new();
            let mut pr_compile: BTreeMap<String, u64> = BTreeMap::new();
            let mut main_compile: BTreeMap<String, u64> = BTreeMap::new();
            for (mode, run) in test.runs.iter() {
                stats.total_runs += 1;
                let (role, key) = classify(mode, suite.matrix);
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

                if matches!(role, Role::Pr | Role::Main | Role::Latest | Role::Solc) {
                    let bytes = run.average_size() + run.average_runtime_size();
                    if bytes != 0 {
                        *by_role_pipeline
                            .entry((role, pipeline_of(mode)))
                            .or_default() += bytes;
                    }
                }

                if !run.compilation_time.is_empty() {
                    let ms = run.average_compilation_time();
                    match role {
                        Role::Pr => *pr_compile.entry(pipeline_of(mode)).or_default() += ms,
                        Role::Main => *main_compile.entry(pipeline_of(mode)).or_default() += ms,
                        _ => {}
                    }
                }
            }

            // Compile time compares PR∩main per pipeline, so the aggregate
            // and the per-project percentages derive from identical data — a
            // project building on only one side is excluded, not counted as
            // zero. One-sided pipelines still get their table column.
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
                    stats.unbaselined_failures += pr.build_failures + pr.test_failures;
                    continue;
                };
                stats.paired_runs += 1;
                // Pre-existing counts cover only the main runs actually
                // compared — a failing run that vanished from the PR side
                // must not inflate them (it surfaces as main-only below).
                stats.baseline_build_failures += main.build_failures;
                stats.baseline_test_failures += main.test_failures;
                let mode = humanize_mode(key);

                for (kind, main_v, pr_v) in [
                    ("build", main.build_failures, pr.build_failures),
                    ("test", main.test_failures, pr.test_failures),
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

                for (kind, pr_v, main_v) in [
                    ("deploy", pr.average_size(), main.average_size()),
                    (
                        "runtime",
                        pr.average_runtime_size(),
                        main.average_runtime_size(),
                    ),
                ] {
                    // A size on only one side has nothing to compare against:
                    // stated apart, never counted as an output diff — a
                    // contract that builds on one toolchain only must not
                    // flip the "Output changed" headline.
                    if (pr_v == 0) != (main_v == 0) {
                        stats.size_one_sided += 1;
                    } else if stats.size.observe(pr_v, main_v) {
                        stats.top_size_movers.push(
                            row_label,
                            format!("{mode}, {kind}").as_str(),
                            main_v,
                            pr_v,
                        );
                    }
                }

                let (pr_gas, main_gas) = (pr.average_gas(), main.average_gas());
                if stats.gas.observe(pr_gas, main_gas) {
                    if suite.gas_is_gate {
                        stats
                            .top_gas_movers
                            .push(row_label, mode.as_str(), main_gas, pr_gas);
                    } else if let Some(pct) = relative_percent(pr_gas, main_gas) {
                        stats.gas_jitter_percents.push(pct.abs());
                    } else {
                        stats.gas_diffs_without_main += 1;
                    }
                }
            }

            for (key, main) in main_runs.iter() {
                if !pr_runs.contains_key(key) {
                    stats.main_orphan_runs += 1;
                    stats.main_orphan_failures += main.build_failures + main.test_failures;
                }
            }
        }

        stats.project_count = projects.len();
        stats
    }

    pub(crate) fn new_failures(&self) -> usize {
        self.new_build_failures + self.new_test_failures
    }

    pub(crate) fn baseline_failures(&self) -> usize {
        self.baseline_build_failures + self.baseline_test_failures
    }

    /// The benchmark had runs but none classified as the PR toolchain — the
    /// naming convention drifted from the declared tables; better a loud
    /// error than a green comment over empty data.
    pub(crate) fn classification_failed(&self) -> bool {
        self.available && self.total_runs > 0 && self.pr_runs_seen == 0
    }

    /// The report parsed but contains no runs at all — a suite that tested
    /// nothing must not render as a clean pass.
    pub(crate) fn is_empty_report(&self) -> bool {
        self.available && self.total_runs == 0
    }
}

/// The relative PR-vs-base percentage, `None` on a zero base — every
/// percentage in the summary comes from here, so zero-base handling cannot
/// drift between columns.
pub(crate) fn relative_percent(pr: u64, base: u64) -> Option<f64> {
    (base != 0).then(|| (pr as f64 - base as f64) / base as f64 * 100.0)
}

/// The median of the given percentages, if any were collected. Even-length
/// input averages the two middle elements — at n=2 the upper-middle would be
/// the maximum, not a median.
pub(crate) fn median(pcts: &[f64]) -> Option<f64> {
    if pcts.is_empty() {
        return None;
    }
    let mut pcts = pcts.to_vec();
    pcts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = pcts.len() / 2;
    Some(if pcts.len().is_multiple_of(2) {
        (pcts[mid - 1] + pcts[mid]) / 2.0
    } else {
        pcts[mid]
    })
}

#[cfg(test)]
mod tests {
    use super::DiffCounter;
    use super::TopMovers;
    use super::median;

    #[test]
    fn median_averages_the_two_middles_for_even_input() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[3.0]), Some(3.0));
        assert_eq!(median(&[1.0, 3.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 30.0]), Some(2.0));
    }

    #[test]
    fn diff_counter_skips_uncollected_pairs_and_sums_deltas() {
        let mut counter = DiffCounter::default();
        assert!(!counter.observe(0, 0));
        assert!(!counter.collected());
        assert!(!counter.observe(100, 100));
        assert!(counter.observe(90, 100));
        assert!(counter.observe(115, 100));
        assert!(counter.collected());
        assert_eq!(counter.cells, 3);
        assert_eq!(counter.diffs, 2);
        assert_eq!(counter.delta, 5);
    }

    #[test]
    fn movers_rank_by_magnitude_regardless_of_direction() {
        let mut movers = TopMovers::default();
        movers.push("small", "legacy", 100, 103);
        movers.push("shrunk", "legacy", 100, 80);
        movers.push("grown", "legacy", 100, 110);
        let ranked = movers.ranked();
        let labels: Vec<&str> = ranked
            .iter()
            .map(|movement| movement.label.as_str())
            .collect();
        assert_eq!(labels, ["shrunk", "grown", "small"]);
    }
}
