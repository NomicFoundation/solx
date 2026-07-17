//!
//! Markdown summary of an integration-test benchmark comparison.
//!
//! Renders the one-comment PR summary the integration workflow posts: the
//! correctness verdict (bytecode size everywhere + solx-tester gas), new
//! failures vs main, and a threshold-gated compile-time tripwire. The verdict
//! is computed here — the single source of truth shared by every suite —
//! instead of parsing the XLSX back offline.
//!
//! `toolchain` interprets mode strings (role and pairing), `stats` reduces
//! each suite's benchmark to numbers, `verdict` turns the numbers into typed
//! decisions, and `render` turns decisions and numbers into markdown.
//!
//! Golden tests pin full rendered comments under `output/summary/fixtures/`;
//! after an intended output change, regenerate them with
//! `UPDATE_SUMMARY_FIXTURES=1 cargo test -p solx-benchmark-converter`.
//!

pub mod compile_aggregate;
pub mod compile_view;
pub mod diff_counter;
pub mod failure_regression;
pub mod failure_regressions;
pub mod failure_verdict;
pub mod gas_change;
pub mod health_issue;
pub mod listing_section;
pub mod movement;
pub mod output_verdict;
pub mod paired_bytes;
pub mod size_change;
pub mod suite_failures;
pub mod suite_row;
pub mod suite_stats;
pub mod summary_template;
pub mod top_movers;
pub mod truncated;

use crate::summary_suite::SummarySuite;

use self::suite_stats::SuiteStats;
use self::summary_template::SummaryTemplate;

///
/// The suites a single PR summary comment is rendered from.
///
pub struct Summary {
    suites: Vec<SummarySuite>,
}

impl Summary {
    /// Collects the suites the workflow fed in.
    pub fn new(suites: Vec<SummarySuite>) -> Self {
        Self { suites }
    }

    /// Renders the full PR summary comment.
    pub fn render(&self) -> String {
        let stats: Vec<SuiteStats> = self.suites.iter().map(SuiteStats::from_suite).collect();
        SummaryTemplate::rendered(&stats)
    }

    /// Whether no suite was fed in — nothing to summarize.
    pub fn is_empty(&self) -> bool {
        self.suites.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::benchmark::Benchmark;
    use crate::benchmark::run_failures::RunFailures;
    use crate::benchmark::test::Test;
    use crate::benchmark::test::input::Input as TestInput;
    use crate::benchmark::test::metadata::Metadata;
    use crate::benchmark::test::run::Run;
    use crate::benchmark::test::selector::Selector;
    use crate::output::summary::Summary;
    use crate::suite_kind::SuiteKind;
    use crate::suite_outcome::SuiteOutcome;
    use crate::summary_suite::SummarySuite;

    fn render(suites: Vec<SummarySuite>) -> String {
        Summary::new(suites).render()
    }

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

    fn failure_test(project: &str, runs: &[(&str, RunFailures)]) -> (String, Test) {
        let selector = Selector {
            project: project.to_owned(),
            case: None,
            input: None,
        };
        let mut test = Test::new(Metadata::new(selector.clone(), vec![]));
        for (mode, failures) in runs {
            let run = Run {
                failures: Some(*failures),
                ..Default::default()
            };
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    /// One input of a case, as the tester's native report emits them: a deploy
    /// and a call per function all share the case and differ only by input.
    fn input_test(case: &str, input: TestInput, runs: &[(&str, u64)]) -> (String, Test) {
        let selector = Selector {
            project: "solx-tester".to_owned(),
            case: Some(case.to_owned()),
            input: Some(input),
        };
        let mut test = Test::new(Metadata::new(selector.clone(), vec![]));
        for (mode, gas) in runs {
            let mut run = Run::default();
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

    /// Merges the given tests by selector, like the real report ingestion
    /// does — a project's failure and compile-time entries share one key.
    fn suite(kind: SuiteKind, tests: Vec<(String, Test)>) -> SummarySuite {
        let mut benchmark = Benchmark::default();
        for (name, test) in tests {
            let entry = benchmark
                .tests
                .entry(name)
                .or_insert_with(|| Test::new(test.metadata.clone()));
            for (mode, run) in test.runs {
                entry
                    .runs
                    .entry(mode)
                    .or_default()
                    .extend(&run)
                    .expect("run merging");
            }
        }
        SummarySuite {
            kind,
            benchmark: Some(benchmark),
            report_url: None,
            outcome: SuiteOutcome::Success,
        }
    }

    fn unavailable(kind: SuiteKind) -> SummarySuite {
        SummarySuite {
            kind,
            benchmark: None,
            report_url: None,
            outcome: SuiteOutcome::Success,
        }
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
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("contracts built by both only"), "{out}");
        assert!(out.contains("| Foundry | legacy | +5.7% | — |"), "{out}");
    }

    #[test]
    fn compile_aggregate_pairs_pr_and_main() {
        // Project b builds only on the PR side — it must be excluded from
        // the aggregate, not skew it as a one-sided +9000 ms.
        let tests = vec![
            compile_test(
                "a",
                &[("02.solx-main-legacy", 1_000), ("03.solx-legacy", 1_030)],
            ),
            compile_test("b", &[("03.solx-legacy", 9_000)]),
        ];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(
            out.contains("| Foundry · 2 proj | +3.0% / +3.0% |"),
            "{out}"
        );
    }

    #[test]
    fn one_sided_compile_pipeline_keeps_its_column() {
        // viaIR compiles on the PR side only: no aggregate exists, but the
        // column must appear with an empty cell instead of vanishing.
        let tests = vec![
            compile_test(
                "a",
                &[("02.solx-main-legacy", 1_000), ("03.solx-legacy", 1_020)],
            ),
            compile_test("b", &[("03.solx-viaIR", 9_000)]),
        ];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(
            out.contains("| Suite | legacy (agg / median) | viaIR (agg / median) |"),
            "{out}"
        );
        assert!(
            out.contains("| Foundry · 2 proj | +2.0% / +2.0% | — |"),
            "{out}"
        );
    }

    #[test]
    fn compile_time_without_a_baseline_makes_no_claim() {
        // Every pipeline ran on the PR side only — reachable whenever the
        // main-side build fails, since its step is continue-on-error. The
        // table is all dashes, so "within noise" would be a reassurance drawn
        // from zero comparisons; worse, dropping the data entirely would make
        // the section vanish and claim nothing at all.
        let tests = vec![compile_test("a", &[("03.solx-legacy", 9_000)])];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("| Foundry | — |"), "{out}");
        assert!(out.contains("_No paired compile-time data"), "{out}");
        assert!(!out.contains("Within noise"), "{out}");
    }

    #[test]
    fn compile_improvements_are_not_sirened() {
        let tests = vec![compile_test(
            "a",
            &[("02.solx-main-legacy", 1_000), ("03.solx-legacy", 700)],
        )];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("| **-30.0%** / -30.0% |"), "{out}");
        assert!(
            out.contains("**Project outliers (≥15%):** `a` legacy **-30.0%**"),
            "{out}"
        );
        assert!(!out.contains("⚠️"), "{out}");
    }

    #[test]
    fn unknown_codegen_token_is_a_loud_harness_error() {
        // A new tester codegen letter must not silently group data under a
        // bogus solc-version pipeline column.
        let tester = suite(
            SuiteKind::Tester,
            vec![compile_test(
                "solx-tester",
                &[
                    ("00.solx-main-solx-L-M3B3-0.8.34", 1_000),
                    ("01.solx-solx-L-M3B3-0.8.34", 1_010),
                ],
            )],
        );
        let out = render(vec![tester]);
        assert!(
            out.contains("❌ **Harness error** — solx-tester: no recognized pipeline token in:"),
            "{out}"
        );
        assert!(!out.contains("0.8.34 (agg / median)"), "{out}");
    }

    #[test]
    fn unrecognized_pipeline_modes_are_listed_but_bounded() {
        // A new codegen letter makes every mode in the suite unrecognized, and
        // a real tester run carries hundreds — the harness-error line must name
        // a few and count the rest, never dump the lot into the comment.
        let modes: Vec<String> = (0..7)
            .map(|index| format!("01.solx-solx-L-M3B3-0.8.3{index}"))
            .collect();
        let runs: Vec<(&str, u64)> = modes.iter().map(|mode| (mode.as_str(), 1_000)).collect();
        let tester = suite(
            SuiteKind::Tester,
            vec![compile_test("solx-tester", runs.as_slice())],
        );
        let out = render(vec![tester]);
        assert!(
            out.contains(
                "❌ **Harness error** — solx-tester: no recognized pipeline token in: \
                 `01.solx-solx-L-M3B3-0.8.30`, `01.solx-solx-L-M3B3-0.8.31`, \
                 `01.solx-solx-L-M3B3-0.8.32`, `01.solx-solx-L-M3B3-0.8.33`, \
                 `01.solx-solx-L-M3B3-0.8.34` (+2 more)."
            ),
            "{out}"
        );
    }

    #[test]
    fn skipped_suite_renders_an_explicit_row() {
        let tester = suite(
            SuiteKind::Tester,
            vec![contract_test(
                "solx-tester",
                "simple/default.sol",
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                    ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                ],
            )],
        );
        let mut foundry = unavailable(SuiteKind::Foundry);
        foundry.outcome = SuiteOutcome::Skipped;
        let out = render(vec![tester, foundry]);
        assert!(
            out.contains("| Foundry | ⚪ did not run | — | — | — |"),
            "{out}"
        );
        assert!(!out.contains("Suite errored"), "{out}");
    }

    #[test]
    fn failed_step_with_data_is_qualified() {
        let mut foundry = suite(
            SuiteKind::Foundry,
            vec![failure_test(
                "p",
                &[
                    ("02.solx-main-legacy", RunFailures::Test(1)),
                    ("03.solx-legacy", RunFailures::Test(1)),
                ],
            )],
        );
        foundry.outcome = SuiteOutcome::Failure;
        let out = render(vec![foundry]);
        assert!(
            out.contains("⚠️ **Suite step failed** — Foundry exited nonzero"),
            "{out}"
        );
    }

    #[test]
    fn one_sided_size_never_flips_the_output_verdict() {
        // The PR builds a contract main could not: nothing common changed,
        // so the headline stays green and the one-sided pair is stated apart.
        let tests = vec![
            contract_test(
                "p",
                "C1",
                &[
                    ("02.solx-main-legacy", 1_000, 0),
                    ("03.solx-legacy", 1_000, 0),
                ],
            ),
            contract_test(
                "p",
                "C2",
                &[("02.solx-main-legacy", 0, 0), ("03.solx-legacy", 22_104, 0)],
            ),
        ];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("✅ **Output-preserving**"), "{out}");
        assert!(out.contains("✅ 0 of 1, ⚪ 1 one-sided"), "{out}");
    }

    #[test]
    fn bytecode_the_pr_stopped_emitting_is_not_excused_as_one_sided() {
        // The mirror of the above: `main` built C2 and the PR emits nothing.
        // Losing 22 KB of compiler output is a regression, not a pair with no
        // baseline to compare against.
        let tests = vec![
            contract_test(
                "p",
                "C1",
                &[
                    ("02.solx-main-legacy", 1_000, 0),
                    ("03.solx-legacy", 1_000, 0),
                ],
            ),
            contract_test(
                "p",
                "C2",
                &[("02.solx-main-legacy", 22_104, 0), ("03.solx-legacy", 0, 0)],
            ),
        ];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("⚠️ **Output changed**"), "{out}");
        assert!(!out.contains("one-sided"), "{out}");
    }

    #[test]
    fn a_single_differing_comparison_among_many_agrees_in_the_singular() {
        // The verb tracks how many differ, not the total compared: one contract
        // changing among three reads "1 of 3 … differs", never "differ".
        let tests = vec![
            contract_test(
                "p",
                "C1",
                &[
                    ("02.solx-main-legacy", 1_000, 0),
                    ("03.solx-legacy", 1_050, 0),
                ],
            ),
            contract_test(
                "p",
                "C2",
                &[
                    ("02.solx-main-legacy", 2_000, 0),
                    ("03.solx-legacy", 2_000, 0),
                ],
            ),
            contract_test(
                "p",
                "C3",
                &[
                    ("02.solx-main-legacy", 3_000, 0),
                    ("03.solx-legacy", 3_000, 0),
                ],
            ),
        ];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(out.contains("1 of 3 size comparisons differs"), "{out}");
    }

    #[test]
    fn main_orphan_runs_are_surfaced_and_not_counted_as_pre_existing() {
        // Main still runs a failing mode the PR side lost: its 7 failures
        // must not inflate "pre-existing", and the shrunken comparison set
        // must be called out instead of silently passing.
        let tests = vec![failure_test(
            "flaky-project",
            &[
                ("02.solx-main-legacy", RunFailures::Test(2)),
                ("03.solx-legacy", RunFailures::Test(2)),
                ("02.solx-main-viaIR", RunFailures::Test(7)),
            ],
        )];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(
            out.contains(
                "⚠️ **Missing on PR** — Foundry: 1 run (7 failures) exists only on `main`"
            ),
            "{out}"
        );
        assert!(out.contains("✅ 0 (2 pre-existing)"), "{out}");
    }

    #[test]
    fn tests_that_never_ran_are_not_a_measured_zero() {
        // A toolchain whose build failed has no test entry: the runner pushes
        // its build failures and skips the test report entirely. That absence
        // must not read as a clean baseline the PR regressed against.
        let tests = vec![failure_test(
            "p",
            &[
                ("02.solx-main-legacy", RunFailures::Build(2)),
                ("03.solx-legacy", RunFailures::Test(3)),
            ],
        )];
        let out = render(vec![suite(SuiteKind::Foundry, tests)]);
        assert!(!out.contains("test failures 0 → 3"), "{out}");
        assert!(!out.contains("+3 test"), "{out}");
        assert!(out.contains("✅ **No new failures**"), "{out}");
    }

    #[test]
    fn gas_movers_name_the_input_they_measured() {
        // A deploy and a call of the same case are two rows sharing one label:
        // without the input, both bullets read identically and the reviewer
        // cannot tell which one regressed. Deploy stays unmarked — the Foundry
        // reports name their deployer after the contract it already carries.
        let tests = vec![
            input_test(
                "delete_struct.sol",
                TestInput::Deployer {
                    contract_identifier: "C".to_owned(),
                },
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 85_899),
                    ("01.solx-solx-E-M3B3-0.8.34", 85_902),
                ],
            ),
            input_test(
                "delete_struct.sol",
                TestInput::Runtime {
                    input_index: 1,
                    name: "transfer(address)".to_owned(),
                },
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 12_000),
                    ("01.solx-solx-E-M3B3-0.8.34", 12_400),
                ],
            ),
        ];
        let out = render(vec![suite(SuiteKind::Tester, tests)]);
        assert!(
            out.contains("`delete_struct.sol` [EVMLA M3B3 0.8.34]"),
            "{out}"
        );
        assert!(
            out.contains("`delete_struct.sol[transfer(address):1]` [EVMLA M3B3 0.8.34]"),
            "{out}"
        );
    }

    #[test]
    fn one_sided_gas_is_not_averaged_into_jitter() {
        // Gas between a measurement and its absence has no meaningful
        // percentage, in either direction: 0 → 50,000 must not be understated
        // by an empty-median "<0.1%", and 50,000 → 0 must not enter the jitter
        // population as a fabricated 100% sample.
        for runs in [
            [
                ("02.solx-main-legacy", 100, 0),
                ("03.solx-legacy", 100, 50_000),
            ],
            [
                ("02.solx-main-legacy", 100, 50_000),
                ("03.solx-legacy", 100, 0),
            ],
        ] {
            let tests = vec![contract_test("p", "C", runs.as_slice())];
            let out = render(vec![suite(SuiteKind::Foundry, tests)]);
            assert!(out.contains("⚪ 1 one-sided (not gated)"), "{out}");
            assert!(!out.contains("jitter"), "{out}");
        }
    }

    #[test]
    fn empty_report_is_a_loud_health_issue() {
        let out = render(vec![suite(SuiteKind::Foundry, vec![])]);
        assert!(
            out.contains("❌ **Suite empty** — Foundry's report contains no runs."),
            "{out}"
        );
        assert!(out.contains("| Foundry | ❌ empty report |"), "{out}");
        assert!(!out.contains("✅ **No new failures**"), "{out}");
    }

    /// Compares a rendered comment against its golden fixture. Set
    /// `UPDATE_SUMMARY_FIXTURES=1` to regenerate after an intended change.
    fn assert_matches_fixture(name: &str, rendered: &str) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/output/summary/fixtures")
            .join(format!("{name}.md"));
        if std::env::var_os("UPDATE_SUMMARY_FIXTURES").is_some() {
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
        // The workflow wraps the tester benchmark in a single "solx-tester"
        // project before conversion, so its row never shows a project count.
        let mut tester = suite(
            SuiteKind::Tester,
            vec![
                contract_test(
                    "solx-tester",
                    "test/libsolidity/semanticTests/structs/delete_struct.sol",
                    &[
                        ("00.solx-main-solx-E-M3B3-0.8.34", 214, 85_899),
                        ("01.solx-solx-E-M3B3-0.8.34", 214, 85_899),
                        ("00.solx-main-solx-Y-M3B3-0.8.34", 198, 85_412),
                        ("01.solx-solx-Y-M3B3-0.8.34", 198, 85_412),
                    ],
                ),
                contract_test(
                    "solx-tester",
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
            SuiteKind::Foundry,
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
                    &[
                        ("02.solx-main-legacy", RunFailures::Test(3)),
                        ("03.solx-legacy", RunFailures::Test(3)),
                    ],
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
            SuiteKind::Hardhat,
            vec![
                failure_test(
                    "ethers-project",
                    &[
                        ("02.solx-main-legacy", RunFailures::Test(2)),
                        ("03.solx-legacy", RunFailures::Test(2)),
                    ],
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

        let out = render(vec![tester, foundry, hardhat]);
        assert_matches_fixture("standard-output-preserving", &out);
    }

    /// Size and gated-gas differences: the warning verdict, inline movers,
    /// and the "+N more" truncation past `MAX_LISTED`.
    #[test]
    fn fixture_output_changed() {
        let tester = suite(
            SuiteKind::Tester,
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
        // Runtime code sizes are comparison cells of their own; this pair's
        // +48 B outranks every deploy diff, so the top mover says "runtime".
        let (_, c0) = &mut foundry_tests[0];
        for (mode, runtime_size) in [("02.solx-main-legacy", 2_000), ("03.solx-legacy", 2_048)] {
            c0.runs
                .get_mut(mode)
                .expect("mode")
                .runtime_size
                .push(runtime_size);
        }
        let foundry = suite(SuiteKind::Foundry, foundry_tests);

        let out = render(vec![tester, foundry]);
        assert_matches_fixture("output-changed", &out);
    }

    /// Build and test regressions: the red verdict and the inline listing of
    /// regressed projects, including the shapes a build failure produces — no
    /// test count on the failed side, and a `main` build failure that leaves
    /// its PR counterpart nothing to regress against.
    #[test]
    fn fixture_new_failures() {
        let foundry = suite(
            SuiteKind::Foundry,
            vec![
                failure_test(
                    "uniswap-v4",
                    &[
                        ("02.solx-main-legacy", RunFailures::Test(5)),
                        ("03.solx-legacy", RunFailures::Build(1)),
                    ],
                ),
                failure_test(
                    "solady",
                    &[
                        ("02.solx-main-viaIR", RunFailures::Build(2)),
                        ("03.solx-viaIR", RunFailures::Test(3)),
                    ],
                ),
                failure_test(
                    "op",
                    &[
                        ("02.solx-main-legacy", RunFailures::Test(4)),
                        ("03.solx-legacy", RunFailures::Test(4)),
                    ],
                ),
                failure_test(
                    "aave",
                    &[
                        ("02.solx-main-legacy", RunFailures::Test(0)),
                        ("03.solx-legacy", RunFailures::Build(1)),
                    ],
                ),
                failure_test(
                    "morpho",
                    &[
                        ("02.solx-main-viaIR", RunFailures::Test(1)),
                        ("03.solx-viaIR", RunFailures::Test(2)),
                    ],
                ),
            ],
        );
        let out = render(vec![foundry]);
        assert_matches_fixture("new-failures", &out);
    }

    /// Every harness-degradation signal at once: an errored suite, toolchain
    /// naming that matches nothing, a foreign run next to healthy PR data,
    /// and runs without a `main` baseline.
    #[test]
    fn fixture_degraded_harness() {
        let foundry = suite(
            SuiteKind::Foundry,
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
            SuiteKind::Hardhat,
            vec![failure_test(
                "hh-project",
                &[
                    ("03.solx-legacy", RunFailures::Test(5)),
                    ("04.mason-legacy", RunFailures::Test(0)),
                ],
            )],
        );
        // The errored suite keeps its report link: the XLSX can outlive a
        // benchmark-JSON write failure.
        let mut tester = unavailable(SuiteKind::Tester);
        tester.report_url = Some("https://example.com/artifacts/tester".to_owned());
        let out = render(vec![tester, foundry, hardhat]);
        assert_matches_fixture("degraded-harness", &out);
    }

    /// The full-matrix run: solc and released-solx baselines plus enough
    /// compile-time project outliers to truncate past `MAX_LISTED`.
    #[test]
    fn fixture_full_matrix() {
        let tester = suite(
            SuiteKind::Tester,
            vec![contract_test(
                "solx-tester",
                "simple/default.sol",
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                    ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                ],
            )],
        );
        let mut foundry_tests = vec![
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
        ];
        for index in 0..5u64 {
            foundry_tests.push(compile_test(
                format!("proj-{index}").as_str(),
                &[
                    ("02.solx-main-legacy", 10_000),
                    ("03.solx-legacy", 11_600 + index * 100),
                ],
            ));
        }
        let foundry = suite(SuiteKind::Foundry, foundry_tests);
        let out = render(vec![tester, foundry]);
        assert_matches_fixture("full-matrix", &out);
    }

    /// The ungated gas shapes: a jitter median below the display floor and a
    /// suite whose collected gas is identical everywhere.
    #[test]
    fn fixture_gas_jitter() {
        let foundry = suite(
            SuiteKind::Foundry,
            vec![
                contract_test(
                    "solady",
                    "src/A.sol:A",
                    &[
                        ("02.solx-main-legacy", 100, 1_000_000),
                        ("03.solx-legacy", 100, 1_000_200),
                    ],
                ),
                contract_test(
                    "solady",
                    "src/B.sol:B",
                    &[
                        ("02.solx-main-legacy", 100, 500_000),
                        ("03.solx-legacy", 100, 500_100),
                    ],
                ),
            ],
        );
        let hardhat = suite(
            SuiteKind::Hardhat,
            vec![contract_test(
                "hh-project",
                "contracts/C.sol:C",
                &[
                    ("02.solx-main-legacy", 200, 42_000),
                    ("03.solx-legacy", 200, 42_000),
                ],
            )],
        );
        let out = render(vec![foundry, hardhat]);
        assert_matches_fixture("gas-jitter", &out);
    }
}
