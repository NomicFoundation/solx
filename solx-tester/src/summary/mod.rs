//!
//! `solx` tester summary.
//!

pub mod benchmark_adapters;
pub mod element;

use std::sync::Arc;
use std::sync::Mutex;

use benchmark_adapters::mode::ModeInfo;
use colored::Colorize;

use crate::test::case::input::output::Output;
use crate::test::description::TestDescription;
use crate::toolchain::Toolchain;

use self::element::Element;
use self::element::outcome::Outcome;
use self::element::outcome::passed_variant::PassedVariant;

///
/// `solx` tester summary.
///
#[derive(Debug)]
pub struct Summary {
    /// The summary elements.
    elements: Vec<Element>,
    /// The output verbosity.
    verbosity: bool,
    /// Whether the output is suppressed.
    quiet: bool,
    /// The passed tests counter.
    passed: usize,
    /// The failed tests counter.
    failed: usize,
    /// The invalid tests counter.
    invalid: usize,
    /// The ignored tests counter.
    ignored: usize,
}

impl Summary {
    /// The elements vector default capacity.
    pub const ELEMENTS_INITIAL_CAPACITY: usize = 1024 * 4096;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(verbosity: bool, quiet: bool) -> Self {
        Self {
            elements: Vec::with_capacity(Self::ELEMENTS_INITIAL_CAPACITY),
            verbosity,
            quiet,
            passed: 0,
            failed: 0,
            invalid: 0,
            ignored: 0,
        }
    }

    ///
    /// Whether the test run has been successful.
    ///
    pub fn is_successful(&self) -> bool {
        for element in self.elements.iter() {
            match element.outcome {
                Outcome::Passed { .. } => continue,
                Outcome::Failed { .. } => return false,
                Outcome::Invalid { .. } => return false,
                Outcome::Ignored => continue,
            }
        }

        true
    }

    ///
    /// Returns the benchmark structure.
    ///
    pub fn benchmark(
        &self,
        toolchain: Toolchain,
    ) -> anyhow::Result<solx_benchmark_converter::Benchmark> {
        if let Toolchain::SolxMlir = toolchain {
            anyhow::bail!("The benchmarking is not supported for the SolxMlir toolchain.")
        }

        let mut benchmark = solx_benchmark_converter::Benchmark::default();

        for Element {
            test_description:
                TestDescription {
                    group,
                    mode,
                    selector,
                },
            outcome,
        } in self.elements.iter()
        {
            let (deploy_size, runtime_size, gas) = match outcome {
                Outcome::Passed {
                    variant:
                        PassedVariant::Deploy {
                            deploy_size,
                            runtime_size,
                            gas,
                        },
                    ..
                } => (Some(*deploy_size), Some(*runtime_size), *gas),
                Outcome::Passed {
                    variant: PassedVariant::Runtime { gas },
                    ..
                } => (None, None, *gas),
                _ => continue,
            };

            let test_name = selector.to_string();

            let tags: Vec<String> = group.iter().cloned().collect();

            let ModeInfo {
                codegen,
                optimizations,
                version,
            } = mode
                .clone()
                .expect("The compiler mode is missing from description.")
                .into();

            let run = benchmark
                .tests
                .entry(test_name)
                .or_insert(solx_benchmark_converter::BenchmarkTest::new(
                    solx_benchmark_converter::BenchmarkTestMetadata::new(
                        selector.clone().into(),
                        tags,
                    ),
                ))
                .toolchain_groups
                .entry(toolchain.to_string())
                .or_insert(Default::default())
                .codegen_groups
                .entry(Some(codegen))
                .or_insert(Default::default())
                .versioned_groups
                .entry(Some(version))
                .or_insert(Default::default())
                .executables
                .entry(Some(optimizations))
                .or_default();
            if let Some(deploy_size) = deploy_size {
                run.run.size.push(deploy_size);
            }
            if let Some(runtime_size) = runtime_size {
                run.run.runtime_size.push(runtime_size);
            }
            run.run.gas.push(gas);
        }
        Ok(benchmark)
    }

    ///
    /// Wraps data into a thread-safe shared reference.
    ///
    pub fn wrap(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    ///
    /// Extracts the data from the thread-safe shared reference.
    ///
    pub fn unwrap_arc(summary: Arc<Mutex<Self>>) -> Self {
        Arc::try_unwrap(summary)
            .expect("Last shared reference")
            .into_inner()
            .expect("Last shared reference")
    }

    ///
    /// Adds a passed outcome of a deploy call.
    ///
    pub fn passed_deploy(
        summary: Arc<Mutex<Self>>,
        test: TestDescription,
        deploy_size: u64,
        runtime_size: u64,
        gas: u64,
    ) {
        let passed_variant = PassedVariant::Deploy {
            deploy_size,
            runtime_size,
            gas,
        };
        Self::passed(summary, test, passed_variant);
    }

    ///
    /// Adds a passed outcome of an ordinary call.
    ///
    pub fn passed_runtime(summary: Arc<Mutex<Self>>, test: TestDescription, gas: u64) {
        let passed_variant = PassedVariant::Runtime { gas };
        Self::passed(summary, test, passed_variant);
    }

    ///
    /// Adds a passed outcome of a special call, like `storageEmpty` or `balance`.
    ///
    pub fn passed_special(summary: Arc<Mutex<Self>>, test: TestDescription) {
        let passed_variant = PassedVariant::Special;
        Self::passed(summary, test, passed_variant);
    }

    ///
    /// Adds a failed outcome.
    ///
    pub fn failed(
        summary: Arc<Mutex<Self>>,
        test: TestDescription,
        expected: Output,
        found: Output,
        calldata: Vec<u8>,
    ) {
        let element = Element::new(test, Outcome::failed(expected, found, calldata));
        summary.lock().expect("Sync").push_element(element);
    }

    ///
    /// Adds an invalid outcome.
    ///
    pub fn invalid<S>(summary: Arc<Mutex<Self>>, test: TestDescription, error: S)
    where
        S: ToString,
    {
        let element = Element::new(test, Outcome::invalid(error));
        summary.lock().expect("Sync").push_element(element);
    }

    ///
    /// Adds an ignored outcome.
    ///
    pub fn ignored(summary: Arc<Mutex<Self>>, test: TestDescription) {
        let element = Element::new(test.with_erased_mode(), Outcome::ignored());
        summary.lock().expect("Sync").push_element(element);
    }

    ///
    /// The unified function for passed outcomes.
    ///
    fn passed(summary: Arc<Mutex<Self>>, test: TestDescription, passed_variant: PassedVariant) {
        let group = test.group.clone();
        let element = Element::new(test, Outcome::passed(group, passed_variant));
        summary.lock().expect("Sync").push_element(element);
    }

    ///
    /// Pushes an element to the summary, printing it.
    ///
    fn push_element(&mut self, element: Element) {
        if let Some(string) = element.print(self.verbosity) {
            println!("{string}");
        }

        let is_executed = match element.outcome {
            Outcome::Passed { .. } => {
                self.passed += 1;
                true
            }
            Outcome::Failed { .. } => {
                self.failed += 1;
                true
            }
            Outcome::Invalid { .. } => {
                self.invalid += 1;
                true
            }
            Outcome::Ignored => {
                self.ignored += 1;
                false
            }
        };

        if is_executed {
            let milestone = if self.verbosity {
                usize::pow(10, 3)
            } else {
                usize::pow(10, 5)
            };

            if (self.passed + self.failed + self.invalid).is_multiple_of(milestone) {
                println!("{self}");
            }
        }

        self.elements.push(element);
    }
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.quiet {
            return Ok(());
        }

        writeln!(
            f,
            "╔═══════════════════╡ INTEGRATION TESTING ╞════════════════════╗"
        )?;
        writeln!(
            f,
            "║                                                              ║"
        )?;
        writeln!(
            f,
            "║     {:7}                                   {:10}     ║",
            "PASSED".green(),
            self.passed.to_string().green(),
        )?;
        writeln!(
            f,
            "║     {:7}                                   {:10}     ║",
            "FAILED".bright_red(),
            self.failed.to_string().bright_red(),
        )?;
        writeln!(
            f,
            "║     {:7}                                   {:10}     ║",
            "INVALID".red(),
            self.invalid.to_string().red(),
        )?;
        writeln!(
            f,
            "║     {:7}                                   {:10}     ║",
            "IGNORED".bright_black(),
            self.ignored.to_string().bright_black(),
        )?;
        writeln!(
            f,
            "║               {:10} TESTS MILESTONE                     ║",
            self.passed + self.failed + self.invalid,
        )?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════╝"
        )?;

        Ok(())
    }
}
