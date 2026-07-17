//!
//! XLSX output format for benchmark data.
//!

pub mod worksheet;

use std::collections::HashMap;

use crate::benchmark::Benchmark;
use crate::output::comparison::Comparison;

use self::worksheet::Worksheet;

///
/// XLSX output format for benchmark data.
///
pub struct Xlsx {
    /// Worksheet for build failures report.
    pub build_failures_worksheet: Worksheet,
    /// Worksheet for test failures report.
    pub test_failures_worksheet: Worksheet,
    /// Worksheet for runtime fee measurements.
    pub runtime_fee_worksheet: Worksheet,
    /// Worksheet for deployment fee measurements.
    pub deploy_fee_worksheet: Worksheet,
    /// Worksheet for runtime bytecode size measurements.
    pub runtime_size_worksheet: Worksheet,
    /// Worksheet for deploy bytecode size measurements.
    pub deploy_size_worksheet: Worksheet,
    /// Worksheet for compilation time measurements.
    pub compilation_time_worksheet: Worksheet,
    /// Worksheet for testing time measurements.
    pub testing_time_worksheet: Worksheet,

    /// Toolchain identifiers.
    pub toolchains: Vec<String>,
    /// Toolchain indexes used to allocate columns.
    pub toolchain_ids: HashMap<String, u16>,
}

impl Xlsx {
    ///
    /// Creates a new XLSX workbook.
    ///
    pub fn new() -> anyhow::Result<Self> {
        let project_header = ("Project", 15);
        let contract_header = ("Contract", 60);
        let function_header = ("Function", 40);

        let build_failures_worksheet = Worksheet::new("Build Failures", vec![project_header])?;
        let test_failures_worksheet = Worksheet::new("Test Failures", vec![project_header])?;
        let runtime_fee_worksheet = Worksheet::new(
            "Runtime Gas",
            vec![project_header, contract_header, function_header],
        )?;
        let deploy_fee_worksheet =
            Worksheet::new("Deploy Gas", vec![project_header, contract_header])?;
        let runtime_size_worksheet =
            Worksheet::new("Runtime Size", vec![project_header, contract_header])?;
        let deploy_size_worksheet =
            Worksheet::new("Deploy Size", vec![project_header, contract_header])?;
        let compilation_time_worksheet = Worksheet::new("Compilation Time", vec![project_header])?;
        let testing_time_worksheet = Worksheet::new("Testing Time", vec![project_header])?;

        Ok(Self {
            build_failures_worksheet,
            test_failures_worksheet,
            runtime_fee_worksheet,
            deploy_fee_worksheet,
            runtime_size_worksheet,
            deploy_size_worksheet,
            compilation_time_worksheet,
            testing_time_worksheet,

            toolchains: Vec::with_capacity(8),
            toolchain_ids: HashMap::with_capacity(8),
        })
    }

    ///
    /// Allocates a new toolchain ID or returns an existing one.
    ///
    pub fn get_toolchain_id(&mut self, toolchain_name: &str) -> u16 {
        if let Some(toolchain_id) = self.toolchain_ids.get(toolchain_name) {
            return *toolchain_id;
        }

        let toolchain_id = self.toolchain_ids.len() as u16;
        self.toolchain_ids
            .insert(toolchain_name.to_owned(), toolchain_id);
        self.toolchains.push(toolchain_name.to_owned());
        toolchain_id
    }

    ///
    /// Every worksheet, in workbook order — the one enumeration shared by
    /// totals, diffs, and finalization.
    ///
    fn worksheets_mut(&mut self) -> [&mut Worksheet; 8] {
        [
            &mut self.build_failures_worksheet,
            &mut self.test_failures_worksheet,
            &mut self.runtime_fee_worksheet,
            &mut self.deploy_fee_worksheet,
            &mut self.runtime_size_worksheet,
            &mut self.deploy_size_worksheet,
            &mut self.compilation_time_worksheet,
            &mut self.testing_time_worksheet,
        ]
    }

    ///
    /// Returns the final workbook with all non-empty worksheets.
    ///
    /// Worksheets without data rows are dropped: some suites legitimately
    /// never produce certain measurements (e.g. Hardhat collects no gas or
    /// size), and an empty sheet with dangling comparison columns reads as
    /// broken data rather than absent data. An all-empty workbook is kept
    /// as-is, since XLSX requires at least one worksheet.
    ///
    pub fn finalize(self) -> rust_xlsxwriter::Workbook {
        let worksheets = [
            self.build_failures_worksheet,
            self.test_failures_worksheet,
            self.runtime_fee_worksheet,
            self.deploy_fee_worksheet,
            self.runtime_size_worksheet,
            self.deploy_size_worksheet,
            self.compilation_time_worksheet,
            self.testing_time_worksheet,
        ];

        let mut workbook = rust_xlsxwriter::Workbook::new();
        let all_empty = worksheets.iter().all(|worksheet| worksheet.rows.is_empty());
        for worksheet in worksheets {
            if all_empty || !worksheet.rows.is_empty() {
                workbook.push_worksheet(worksheet.into_inner());
            }
        }
        workbook
    }
}

impl TryFrom<(Benchmark, Vec<Comparison>)> for Xlsx {
    type Error = anyhow::Error;

    fn try_from(
        (benchmark, comparisons): (Benchmark, Vec<Comparison>),
    ) -> Result<Self, Self::Error> {
        let mut xlsx = Self::new()?;

        for test in benchmark.tests.into_values() {
            let is_deployer = test
                .metadata
                .selector
                .input
                .as_ref()
                .map(|input| input.is_deploy())
                .unwrap_or_default();
            let project = test.metadata.selector.project;
            let contract = test.metadata.selector.case.as_deref();
            let function = test
                .metadata
                .selector
                .input
                .as_ref()
                .and_then(|input| input.runtime_name());

            for (mode_name, run) in test.runs.into_iter() {
                let toolchain_id = xlsx.get_toolchain_id(mode_name.as_str());
                let mode = mode_name.as_str();
                let project = project.as_str();

                if !run.compilation_time.is_empty() {
                    xlsx.compilation_time_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        run.average_compilation_time(),
                    )?;
                }
                if !run.testing_time.is_empty() {
                    xlsx.testing_time_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        run.average_testing_time(),
                    )?;
                }
                xlsx.build_failures_worksheet.record(
                    mode,
                    toolchain_id,
                    project,
                    None,
                    None,
                    run.build_failures_count() as u64,
                )?;
                if let Some(test_failures) = run.test_failures_count() {
                    xlsx.test_failures_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        test_failures as u64,
                    )?;
                }

                if contract.is_none() && function.is_none() {
                    continue;
                }
                if is_deployer {
                    if test.non_zero_gas_values > 0 {
                        xlsx.deploy_fee_worksheet.record(
                            mode,
                            toolchain_id,
                            project,
                            contract,
                            None,
                            run.average_gas(),
                        )?;
                    }
                } else {
                    xlsx.runtime_fee_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        contract,
                        function,
                        run.average_gas(),
                    )?;
                }
                if !run.size.is_empty() {
                    xlsx.deploy_size_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        contract,
                        None,
                        run.average_size(),
                    )?;
                }
                if !run.runtime_size.is_empty() {
                    xlsx.runtime_size_worksheet.record(
                        mode,
                        toolchain_id,
                        project,
                        contract,
                        None,
                        run.average_runtime_size(),
                    )?;
                }
            }
        }

        let toolchain_count = xlsx.toolchain_ids.len();
        for worksheet in xlsx.worksheets_mut() {
            worksheet.set_totals(toolchain_count)?;
        }

        let comparison_mapping: Vec<(u16, String, u16, String)> = comparisons
            .iter()
            .filter_map(|comparison| {
                let left_id = *xlsx.toolchain_ids.get(comparison.left.as_str())?;
                let right_id = *xlsx.toolchain_ids.get(comparison.right.as_str())?;
                Some((
                    left_id,
                    xlsx.toolchains[left_id as usize].clone(),
                    right_id,
                    xlsx.toolchains[right_id as usize].clone(),
                ))
            })
            .collect();

        for (index, (left_id, left_name, right_id, right_name)) in
            comparison_mapping.into_iter().enumerate()
        {
            for worksheet in xlsx.worksheets_mut() {
                worksheet.set_diffs(
                    left_id,
                    left_name.as_str(),
                    right_id,
                    right_name.as_str(),
                    toolchain_count as u16,
                    index as u16,
                )?;
            }
        }

        Ok(xlsx)
    }
}
