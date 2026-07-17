//!
//! XLSX output format for benchmark data.
//!

pub mod sheet;
pub mod worksheet;

use std::collections::HashMap;

use crate::benchmark::Benchmark;
use crate::comparison::Comparison;
use crate::output::xlsx::sheet::Sheet;

use self::worksheet::Worksheet;

///
/// XLSX output format for benchmark data.
///
pub struct Xlsx {
    /// The worksheets, indexed by `Sheet`.
    worksheets: Vec<Worksheet>,

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
        let mut worksheets = Vec::with_capacity(Sheet::ALL.len());
        for sheet in Sheet::ALL {
            let (name, headers) = sheet.spec();
            worksheets.push(Worksheet::new(name, headers)?);
        }

        Ok(Self {
            worksheets,

            toolchains: Vec::with_capacity(8),
            toolchain_ids: HashMap::with_capacity(8),
        })
    }

    ///
    /// The worksheet a sheet names.
    ///
    fn sheet(&mut self, sheet: Sheet) -> &mut Worksheet {
        &mut self.worksheets[sheet as usize]
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
    /// Returns the final workbook with all non-empty worksheets.
    ///
    /// Worksheets without data rows are dropped: some suites legitimately
    /// never produce certain measurements (e.g. Hardhat collects no gas or
    /// size), and an empty sheet with dangling comparison columns reads as
    /// broken data rather than absent data. An all-empty workbook is kept
    /// as-is, since XLSX requires at least one worksheet.
    ///
    pub fn finalize(self) -> rust_xlsxwriter::Workbook {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let all_empty = self
            .worksheets
            .iter()
            .all(|worksheet| worksheet.rows.is_empty());
        for worksheet in self.worksheets {
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
                    xlsx.sheet(Sheet::CompilationTime).record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        run.average_compilation_time(),
                    )?;
                }
                if !run.testing_time.is_empty() {
                    xlsx.sheet(Sheet::TestingTime).record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        run.average_testing_time(),
                    )?;
                }
                if let Some(build_failures) = run.build_failures_count() {
                    xlsx.sheet(Sheet::BuildFailures).record(
                        mode,
                        toolchain_id,
                        project,
                        None,
                        None,
                        build_failures as u64,
                    )?;
                }
                if let Some(test_failures) = run.test_failures_count() {
                    xlsx.sheet(Sheet::TestFailures).record(
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
                        xlsx.sheet(Sheet::DeployFee).record(
                            mode,
                            toolchain_id,
                            project,
                            contract,
                            None,
                            run.average_gas(),
                        )?;
                    }
                } else {
                    xlsx.sheet(Sheet::RuntimeFee).record(
                        mode,
                        toolchain_id,
                        project,
                        contract,
                        function,
                        run.average_gas(),
                    )?;
                }
                if !run.size.is_empty() {
                    xlsx.sheet(Sheet::DeploySize).record(
                        mode,
                        toolchain_id,
                        project,
                        contract,
                        None,
                        run.average_size(),
                    )?;
                }
                if !run.runtime_size.is_empty() {
                    xlsx.sheet(Sheet::RuntimeSize).record(
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
        for worksheet in xlsx.worksheets.iter_mut() {
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
            for worksheet in xlsx.worksheets.iter_mut() {
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

#[cfg(test)]
mod tests {
    use crate::output::xlsx::Sheet;
    use crate::output::xlsx::Xlsx;

    #[test]
    fn every_sheet_indexes_its_own_worksheet() {
        // `sheet as usize` indexes the worksheets `new` filled in `ALL` order:
        // a variant added mid-enum but appended to `ALL` would silently write
        // every later sheet's data to the wrong tab.
        let mut xlsx = Xlsx::new().expect("workbook creation");
        assert_eq!(xlsx.worksheets.len(), Sheet::ALL.len());
        for (index, sheet) in Sheet::ALL.into_iter().enumerate() {
            assert_eq!(sheet as usize, index, "{sheet:?}");
            let (name, headers) = sheet.spec();
            assert_eq!(xlsx.sheet(sheet).headers, headers, "{name}");
        }
    }
}
