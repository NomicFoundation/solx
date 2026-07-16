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

mod render;
mod stats;
mod toolchain;
mod verdict;

use crate::benchmark::Benchmark;

use self::render::render_summary;
use self::stats::SuiteStats;

pub use self::toolchain::ToolchainMatrix;

///
/// How the suite's workflow step ended — the comment must distinguish a
/// suite that never ran from one that errored, and qualify data written by
/// a step that then failed.
///
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SuiteOutcome {
    /// The step ran to completion.
    #[default]
    Success,
    /// The step ran but exited nonzero — any report it wrote may be partial.
    Failure,
    /// The step never ran (an earlier hard failure); not the suite's fault.
    Skipped,
}

impl SuiteOutcome {
    ///
    /// Parses a GitHub Actions step outcome; anything unrecognized is
    /// conservatively a failure.
    ///
    pub fn from_step_outcome(outcome: Option<&str>) -> Self {
        match outcome {
            None | Some("success") => Self::Success,
            Some("skipped") => Self::Skipped,
            Some(_) => Self::Failure,
        }
    }
}

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
    /// Which toolchain naming matrix the benchmark's mode strings follow.
    pub matrix: ToolchainMatrix,
    /// How the suite's workflow step ended.
    pub outcome: SuiteOutcome,
}

///
/// Renders the full PR summary comment for the given suites.
///
pub fn render(suites: &[SummarySuite]) -> String {
    let stats: Vec<SuiteStats> = suites.iter().map(SuiteStats::from_suite).collect();
    render_summary(&stats)
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
            matrix: matrix_for(label),
            outcome: SuiteOutcome::default(),
        }
    }

    /// The same suite-to-matrix mapping the summary binary applies.
    fn matrix_for(label: &str) -> ToolchainMatrix {
        if label == "solx-tester" {
            ToolchainMatrix::Tester
        } else {
            ToolchainMatrix::Project
        }
    }

    fn unavailable(label: &str) -> SummarySuite {
        SummarySuite {
            label: label.to_owned(),
            report_file: format!("{}-report.xlsx", label.to_lowercase()),
            benchmark: None,
            report_url: None,
            gas_is_gate: false,
            matrix: matrix_for(label),
            outcome: SuiteOutcome::default(),
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
        let out = render(&[suite("Foundry", false, tests)]);
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
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(
            out.contains("| Foundry · 2 proj | +3.0% / +3.0% |"),
            "{out}"
        );
    }

    #[test]
    fn compile_improvements_are_not_sirened() {
        let tests = vec![compile_test(
            "a",
            &[("02.solx-main-legacy", 1_000), ("03.solx-legacy", 700)],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("| **-30.0%** / -30.0% |"), "{out}");
        assert!(
            out.contains("**Project outliers (≥15%):** `a` legacy **-30.0%**"),
            "{out}"
        );
        assert!(!out.contains("⚠️"), "{out}");
    }

    #[test]
    fn skipped_suite_renders_an_explicit_row() {
        // A suite skipped by an earlier hard failure must appear as "did not
        // run" — a partial summary must never look like a complete one.
        let tester = suite(
            "solx-tester",
            true,
            vec![contract_test(
                "solx-tester",
                "simple/default.sol",
                &[
                    ("00.solx-main-solx-E-M3B3-0.8.34", 460, 21_442),
                    ("01.solx-solx-E-M3B3-0.8.34", 460, 21_442),
                ],
            )],
        );
        let mut foundry = unavailable("Foundry");
        foundry.outcome = SuiteOutcome::Skipped;
        let out = render(&[tester, foundry]);
        assert!(
            out.contains("| Foundry | ⚪ did not run | — | — | — |"),
            "{out}"
        );
        assert!(!out.contains("Suite errored"), "{out}");
    }

    #[test]
    fn failed_step_with_data_is_qualified() {
        let mut foundry = suite(
            "Foundry",
            false,
            vec![failure_test(
                "p",
                &[("02.solx-main-legacy", 0, 1), ("03.solx-legacy", 0, 1)],
            )],
        );
        foundry.outcome = SuiteOutcome::Failure;
        let out = render(&[foundry]);
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
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("✅ **Output-preserving**"), "{out}");
        assert!(out.contains("✅ 0 of 1, ⚪ 1 one-sided"), "{out}");
    }

    #[test]
    fn main_orphan_runs_are_surfaced_and_not_counted_as_pre_existing() {
        // Main still runs a failing mode the PR side lost: its 7 failures
        // must not inflate "pre-existing", and the shrunken comparison set
        // must be called out instead of silently passing.
        let tests = vec![failure_test(
            "flaky-project",
            &[
                ("02.solx-main-legacy", 0, 2),
                ("03.solx-legacy", 0, 2),
                ("02.solx-main-viaIR", 0, 7),
            ],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(
            out.contains(
                "⚠️ **Missing on PR** — Foundry: 1 runs (7 failures) exist only on `main`"
            ),
            "{out}"
        );
        assert!(out.contains("✅ 0 (2 pre-existing)"), "{out}");
    }

    #[test]
    fn one_sided_gas_is_not_averaged_into_jitter() {
        // Gas going 0 → 50,000 has no percentage; it must be stated apart,
        // not understated by an empty-median "<0.1%".
        let tests = vec![contract_test(
            "p",
            "C",
            &[
                ("02.solx-main-legacy", 100, 0),
                ("03.solx-legacy", 100, 50_000),
            ],
        )];
        let out = render(&[suite("Foundry", false, tests)]);
        assert!(out.contains("⚪ 1 without `main` gas (not gated)"), "{out}");
    }

    #[test]
    fn empty_report_is_a_loud_health_issue() {
        let out = render(&[suite("Foundry", false, vec![])]);
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
        // Windows checkouts rewrite the fixtures to CRLF (no .gitattributes).
        let expected = expected.replace("\r\n", "\n");
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
            "solx-tester",
            true,
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

    /// Build and test regressions: the red verdict, the inline listing of
    /// regressed projects, and its "+N more" truncation past `MAX_LISTED`.
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
                failure_test(
                    "aave",
                    &[("02.solx-main-legacy", 0, 0), ("03.solx-legacy", 1, 2)],
                ),
                failure_test(
                    "morpho",
                    &[("02.solx-main-viaIR", 0, 1), ("03.solx-viaIR", 0, 2)],
                ),
            ],
        );
        let out = render(&[foundry]);
        assert_matches_fixture("new-failures", &out);
    }

    /// Every harness-degradation signal at once: an errored suite, toolchain
    /// naming that matches nothing, a foreign run next to healthy PR data,
    /// and runs without a `main` baseline.
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
            vec![failure_test(
                "hh-project",
                &[("03.solx-legacy", 0, 5), ("04.mason-legacy", 0, 0)],
            )],
        );
        // The errored suite keeps its report link: the XLSX can outlive a
        // benchmark-JSON write failure.
        let mut tester = unavailable("solx-tester");
        tester.report_url = Some("https://example.com/artifacts/tester".to_owned());
        let out = render(&[tester, foundry, hardhat]);
        assert_matches_fixture("degraded-harness", &out);
    }

    /// The full-matrix run: solc and released-solx baselines plus enough
    /// compile-time project outliers to truncate past `MAX_LISTED`.
    #[test]
    fn fixture_full_matrix() {
        let tester = suite(
            "solx-tester",
            true,
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
        let foundry = suite("Foundry", false, foundry_tests);
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

    /// The ungated gas shapes: a jitter median below the display floor and a
    /// suite whose collected gas is identical everywhere.
    #[test]
    fn fixture_gas_jitter() {
        let foundry = suite(
            "Foundry",
            false,
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
            "Hardhat",
            false,
            vec![contract_test(
                "hh-project",
                "contracts/C.sol:C",
                &[
                    ("02.solx-main-legacy", 200, 42_000),
                    ("03.solx-legacy", 200, 42_000),
                ],
            )],
        );
        let out = render(&[foundry, hardhat]);
        assert_matches_fixture("gas-jitter", &out);
    }
}
