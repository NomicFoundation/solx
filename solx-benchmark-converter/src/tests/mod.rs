//!
//! Crate-internal test suite.
//!
//! Every test-only constructor and assertion helper lives here at the root, so
//! the submodules below carry test functions only.
//!

mod io;
mod listings;
mod summary;
mod toolchain_matrix;
mod utils;
mod verdicts;

use crate::benchmark::Benchmark;
use crate::benchmark::run_failures::RunFailures;
use crate::benchmark::test::Test;
use crate::benchmark::test::input::Input as TestInput;
use crate::benchmark::test::metadata::Metadata;
use crate::benchmark::test::run::Run;
use crate::benchmark::test::selector::Selector;
use crate::output::summary::Summary;
use crate::output::summary::diff_counter::DiffCounter;
use crate::output::summary::suite_stats::SuiteStats;
use crate::suite_kind::SuiteKind;
use crate::suite_outcome::SuiteOutcome;
use crate::summary_suite::SummarySuite;

impl Benchmark {
    /// A benchmark carrying one runtime-input row, for the blacklist test.
    fn insert_test(&mut self, project: &str, contract: &str, function: &str) {
        let selector = Selector {
            project: project.to_owned(),
            case: Some(contract.to_owned()),
            input: Some(TestInput::Runtime {
                input_index: 0,
                name: function.to_owned(),
            }),
        };
        self.tests.insert(
            selector.to_string(),
            Test::new(Metadata::new(selector, vec![])),
        );
    }
}

impl Test {
    /// A contract's runs by mode, each carrying a deploy size and a gas figure.
    fn contract(project: &str, contract: &str, runs: &[(&str, u64, u64)]) -> (String, Self) {
        let selector = Selector {
            project: project.to_owned(),
            case: Some(contract.to_owned()),
            input: None,
        };
        let mut test = Self::new(Metadata::new(selector.clone(), vec![]));
        for (mode, deploy_size, gas) in runs {
            let mut run = Run::default();
            run.size.push(*deploy_size);
            run.gas.push(*gas);
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    /// A project's failure counts by mode.
    fn failure(project: &str, runs: &[(&str, RunFailures)]) -> (String, Self) {
        let selector = Selector {
            project: project.to_owned(),
            case: None,
            input: None,
        };
        let mut test = Self::new(Metadata::new(selector.clone(), vec![]));
        for (mode, failures) in runs {
            let run = Run {
                failures: Some(*failures),
                ..Default::default()
            };
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    /// One input of a case, as the tester's native report emits them: a
    /// deploy and a call per function all share the case and differ only by
    /// input.
    fn input(case: &str, input: TestInput, runs: &[(&str, u64)]) -> (String, Self) {
        let selector = Selector {
            project: "solx-tester".to_owned(),
            case: Some(case.to_owned()),
            input: Some(input),
        };
        let mut test = Self::new(Metadata::new(selector.clone(), vec![]));
        for (mode, gas) in runs {
            let mut run = Run::default();
            run.gas.push(*gas);
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }

    /// A project's compile-time samples by mode.
    fn compile(project: &str, runs: &[(&str, u64)]) -> (String, Self) {
        let selector = Selector {
            project: project.to_owned(),
            case: None,
            input: None,
        };
        let mut test = Self::new(Metadata::new(selector.clone(), vec![]));
        for (mode, ms) in runs {
            let mut run = Run::default();
            run.compilation_time.push(*ms);
            test.runs.insert((*mode).to_owned(), run);
        }
        (selector.to_string(), test)
    }
}

impl SummarySuite {
    /// Merges the given tests by selector, like the real report ingestion
    /// does: a project's failure and compile-time entries share one key.
    fn merged(kind: SuiteKind, tests: Vec<(String, Test)>) -> Self {
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
        Self {
            kind,
            benchmark: Some(benchmark),
            report_url: None,
            outcome: SuiteOutcome::Success,
        }
    }

    /// A suite whose report never arrived.
    fn unavailable(kind: SuiteKind) -> Self {
        Self {
            kind,
            benchmark: None,
            report_url: None,
            outcome: SuiteOutcome::Success,
        }
    }
}

impl Summary {
    /// Renders this summary and compares it against its golden fixture. Set
    /// `UPDATE_SUMMARY_FIXTURES=1` to regenerate after an intended change.
    fn assert_matches_fixture(&self, name: &str) {
        let rendered = self.render();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/tests/fixtures")
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
}

impl SuiteStats {
    /// An available suite carrying only the given label, for the verdict tests.
    fn available(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            available: true,
            ..Self::default()
        }
    }
}

impl DiffCounter {
    /// A counter with the given tallies, for the output-verdict tests.
    fn counted(cells: u64, diffs: u64, delta: i128) -> Self {
        Self {
            cells,
            diffs,
            delta,
        }
    }
}
