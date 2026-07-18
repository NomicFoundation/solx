//!
//! The benchmark representation.
//!

pub mod run_failures;
pub mod test;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use crate::comparison::Comparison;
use crate::input::Input;
use crate::input::build_failures::BuildFailuresReport;
use crate::input::compilation_time::CompilationTimeReport;
use crate::input::foundry_gas::FoundryGasReport;
use crate::input::foundry_size::FoundrySizeReport;
use crate::input::report::Report;
use crate::input::test_failures::TestFailuresReport;
use crate::input::testing_time::TestingTimeReport;
use crate::output::Output;
use crate::output::format::Format;
use crate::output::json::Json;
use crate::suite_kind::SuiteKind;

use self::run_failures::RunFailures;
use self::test::Test;
use self::test::input::Input as TestInput;
use self::test::metadata::Metadata as TestMetadata;
use self::test::selector::Selector as TestSelector;

///
/// The benchmark representation.
///
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Benchmark {
    /// The tests.
    pub tests: BTreeMap<String, Test>,
}

impl Benchmark {
    /// Tests whose measurements are known-meaningless noise: proxy fallbacks
    /// and brutalized multicalls whose gas depends on the delegated payload,
    /// as `(project, contract, function)`. Filtered out of the benchmark
    /// itself so every report, spreadsheet and PR summary alike, agrees on
    /// the data set.
    const BLACKLIST: [(&str, &str, &str); 3] = [
        (
            "aave-v3",
            "lib/solidity-utils/lib/openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts/contracts/proxy/transparent/TransparentUpgradeableProxy.sol:TransparentUpgradeableProxy",
            "fallback()",
        ),
        (
            "solady",
            "test/utils/mocks/MockMulticallable.sol:MockMulticallable",
            "multicallBrutalized(bytes[])",
        ),
        (
            "solady",
            "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
            "fallback()",
        ),
    ];

    ///
    /// Creates a benchmark from multiple inputs.
    ///
    /// # Errors
    ///
    /// Returns an error if extending the benchmark with any input fails.
    ///
    pub fn from_inputs<I: IntoIterator<Item = Input>>(inputs: I) -> anyhow::Result<Self> {
        let mut benchmark = Self::default();
        for input in inputs {
            benchmark.extend(input)?;
        }
        benchmark.remove_zero_deploy_gas();
        benchmark.remove_blacklisted();
        Ok(benchmark)
    }

    ///
    /// Extend the benchmark data with a generic report.
    ///
    /// # Errors
    ///
    /// Returns an error if merging the report into the benchmark fails.
    ///
    pub fn extend(&mut self, input: Input) -> anyhow::Result<()> {
        let toolchain = input.toolchain;
        let project = input.project;
        match input.data {
            Report::Native(report) => {
                self.extend_with_native_report(toolchain, project, report)?;
            }
            Report::FoundryGas(report) => {
                self.extend_with_foundry_gas_report(toolchain, project, report)?;
            }
            Report::FoundrySize(report) => {
                self.extend_with_foundry_size_report(toolchain, project, report)?;
            }
            Report::CompilationTime(compilation_time) => {
                self.extend_with_compilation_time_report(toolchain, project, compilation_time)?;
            }
            Report::TestingTime(testing_time) => {
                self.extend_with_testing_time_report(toolchain, project, testing_time)?;
            }
            Report::BuildFailures(build_failures) => {
                self.extend_with_build_failures_report(toolchain, project, build_failures)?;
            }
            Report::TestFailures(test_failures) => {
                self.extend_with_test_failures_report(toolchain, project, test_failures)?;
            }
        }
        Ok(())
    }

    ///
    /// Extend the benchmark data with a native benchmark report.
    ///
    /// # Errors
    ///
    /// Returns an error if merging a run's measurements fails.
    ///
    pub fn extend_with_native_report(
        &mut self,
        toolchain: String,
        project: String,
        mut report: Benchmark,
    ) -> anyhow::Result<()> {
        report.tests.retain(|name, _| {
            name.starts_with("solx-solidity") || name.starts_with("tests/solidity")
        });

        for (name, test) in report.tests.into_iter() {
            let selector = TestSelector {
                project: project.clone(),
                case: Some(name.split('[').next().unwrap_or("Unknown").to_owned()),
                input: test.metadata.selector.input,
            };
            let name = selector.to_string();

            let existing_test = self
                .tests
                .entry(name)
                .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));

            for (mode, run) in test.runs.into_iter() {
                let mode_key = if mode.starts_with(&toolchain) {
                    mode
                } else {
                    format!("{toolchain}-{mode}")
                };
                existing_test
                    .runs
                    .entry(mode_key)
                    .or_default()
                    .extend(&run)?;
            }
        }

        Ok(())
    }

    ///
    /// Extend the benchmark data with a Foundry gas report.
    ///
    /// # Errors
    ///
    /// Returns an error if the Foundry gas report cannot be merged into the benchmark.
    ///
    pub fn extend_with_foundry_gas_report(
        &mut self,
        toolchain: String,
        project: String,
        foundry_report: FoundryGasReport,
    ) -> anyhow::Result<()> {
        for contract_report in foundry_report.0.into_iter() {
            let selector = TestSelector {
                project: project.clone(),
                case: Some(contract_report.contract.to_owned()),
                input: Some(TestInput::Deployer {
                    contract_identifier: contract_report.contract.to_owned(),
                }),
            };
            let name = selector.to_string();

            let test = self
                .tests
                .entry(name)
                .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
            let run = test.runs.entry(toolchain.clone()).or_default();
            run.gas.push(contract_report.deployment.gas);

            for (index, (function, function_report)) in
                contract_report.functions.into_iter().enumerate()
            {
                let selector = TestSelector {
                    project: project.clone(),
                    case: Some(contract_report.contract.to_owned()),
                    input: Some(TestInput::Runtime {
                        input_index: index + 1,
                        name: function,
                    }),
                };
                let name = selector.to_string();

                let test = self
                    .tests
                    .entry(name)
                    .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
                let run = test.runs.entry(toolchain.clone()).or_default();
                run.gas.push(function_report.mean);
            }
        }

        Ok(())
    }

    ///
    /// Extend the benchmark data with a Foundry size report.
    ///
    /// # Errors
    ///
    /// Returns an error if the Foundry size report cannot be merged into the benchmark.
    ///
    pub fn extend_with_foundry_size_report(
        &mut self,
        toolchain: String,
        project: String,
        foundry_report: FoundrySizeReport,
    ) -> anyhow::Result<()> {
        for (contract_name, contract_report) in foundry_report.0.into_iter() {
            let selector = TestSelector {
                project: project.clone(),
                case: Some(contract_name.clone()),
                input: Some(TestInput::Deployer {
                    contract_identifier: contract_name.clone(),
                }),
            };
            let name = selector.to_string();

            let test = self
                .tests
                .entry(name)
                .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
            let run = test.runs.entry(toolchain.clone()).or_default();
            run.size.push(contract_report.init_size);
            run.runtime_size.push(contract_report.runtime_size);
        }

        Ok(())
    }

    ///
    /// Extend the benchmark data with a compilation time report.
    ///
    /// # Errors
    ///
    /// Returns an error if the compilation time report cannot be merged into the benchmark.
    ///
    pub fn extend_with_compilation_time_report(
        &mut self,
        toolchain: String,
        project: String,
        compilation_time: CompilationTimeReport,
    ) -> anyhow::Result<()> {
        let selector = TestSelector {
            project: project.clone(),
            case: None,
            input: None,
        };
        let name = selector.to_string();

        let test = self
            .tests
            .entry(name)
            .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
        let run = test.runs.entry(toolchain.clone()).or_default();
        run.compilation_time.push(compilation_time.0);

        Ok(())
    }

    ///
    /// Extend the benchmark data with a testing time report.
    ///
    /// # Errors
    ///
    /// Returns an error if the testing time report cannot be merged into the benchmark.
    ///
    pub fn extend_with_testing_time_report(
        &mut self,
        toolchain: String,
        project: String,
        testing_time: TestingTimeReport,
    ) -> anyhow::Result<()> {
        let selector = TestSelector {
            project: project.clone(),
            case: None,
            input: None,
        };
        let name = selector.to_string();

        let test = self
            .tests
            .entry(name)
            .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
        let run = test.runs.entry(toolchain.clone()).or_default();
        run.testing_time.push(testing_time.0);

        Ok(())
    }

    ///
    /// Extend the benchmark data with a build failures report.
    ///
    /// # Errors
    ///
    /// Returns an error if the build failures report cannot be merged into the benchmark.
    ///
    pub fn extend_with_build_failures_report(
        &mut self,
        toolchain: String,
        project: String,
        build_failures: BuildFailuresReport,
    ) -> anyhow::Result<()> {
        let selector = TestSelector {
            project: project.clone(),
            case: None,
            input: None,
        };
        let name = selector.to_string();

        let test = self
            .tests
            .entry(name)
            .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
        let run = test.runs.entry(toolchain.clone()).or_default();
        run.failures = Some(RunFailures::Build(build_failures.0));

        Ok(())
    }

    ///
    /// Extend the benchmark data with a test failures report.
    ///
    /// # Errors
    ///
    /// Returns an error if the test failures report cannot be merged into the benchmark.
    ///
    pub fn extend_with_test_failures_report(
        &mut self,
        toolchain: String,
        project: String,
        test_failures: TestFailuresReport,
    ) -> anyhow::Result<()> {
        let selector = TestSelector {
            project: project.clone(),
            case: None,
            input: None,
        };
        let name = selector.to_string();

        let test = self
            .tests
            .entry(name)
            .or_insert_with(|| Test::new(TestMetadata::new(selector, vec![])));
        let run = test.runs.entry(toolchain.clone()).or_default();
        run.failures = Some(RunFailures::Test(test_failures.0));

        Ok(())
    }

    ///
    /// Removes tests with zero deployment gas, that are supposed to be non-deployable contracts.
    ///
    pub fn remove_zero_deploy_gas(&mut self) {
        self.tests.retain(|_, test| {
            if test.runs.is_empty() {
                return false;
            }
            if !test.is_deploy() {
                return true;
            }
            test.non_zero_gas_values = test
                .runs
                .values()
                .filter(|run| run.average_gas() != 0)
                .count();
            test.runs.values().any(|run| {
                run.average_size() != 0 || run.average_runtime_size() != 0 || run.average_gas() != 0
            })
        });
    }

    ///
    /// Removes the tests blacklisted as measurement noise.
    ///
    pub fn remove_blacklisted(&mut self) {
        self.tests.retain(|_, test| {
            let selector = &test.metadata.selector;
            let contract = selector.case.as_deref();
            let function = selector
                .input
                .as_ref()
                .and_then(|input| input.runtime_name());
            !Self::BLACKLIST
                .iter()
                .any(|(project_b, contract_b, function_b)| {
                    selector.project.as_str() == *project_b
                        && contract == Some(*contract_b)
                        && function == Some(*function_b)
                })
        });
    }

    ///
    /// The distinct toolchain columns the benchmark carries: the run-mode keys
    /// across every test, which are the diff comparisons' left/right names.
    ///
    pub fn toolchains(&self) -> BTreeSet<String> {
        self.tests
            .values()
            .flat_map(|test| test.runs.keys().cloned())
            .collect()
    }

    ///
    /// Writes this benchmark's JSON and XLSX reports for the given suite into
    /// the directory: the JSON feeds the summary comment, the XLSX is the
    /// uploaded artifact. The suite kind supplies the file names.
    ///
    /// # Errors
    ///
    /// Returns an error if the output directory cannot be created or a report
    /// file cannot be written.
    ///
    pub fn write_reports(
        self,
        comparisons: Vec<Comparison>,
        kind: SuiteKind,
        output_directory: &Path,
    ) -> anyhow::Result<()> {
        std::fs::create_dir_all(output_directory).map_err(|error| {
            anyhow::anyhow!(
                "{} output directory {output_directory:?} creation: {error}",
                kind.label()
            )
        })?;
        Output::from(Json::from(&self))
            .write_to_file(output_directory.join(kind.benchmark_file()))?;
        let report: Output = (self, comparisons, Format::Xlsx).try_into()?;
        report.write_to_file(output_directory.join(kind.report_file()))?;
        Ok(())
    }
}

impl TryFrom<&Path> for Benchmark {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} reading: {error}"))?;
        let json: Self = serde_json::from_str(text.as_str())
            .map_err(|error| anyhow::anyhow!("Benchmark file {path:?} parsing: {error}"))?;
        Ok(json)
    }
}
