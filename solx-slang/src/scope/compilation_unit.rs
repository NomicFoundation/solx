//!
//! The compilation unit scope: the standard-JSON input's settings every definition of the unit is
//! lowered under. The definitions lower on the global thread pool, the costliest first.
//!

use std::cell::OnceCell;
use std::cmp::Reverse;
use std::collections::BTreeMap;

use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationUnit;

use solx_standard_json::OutputError;
use solx_standard_json::output::contract::Contract;
use solx_standard_json::output::error::source_location::SourceLocation;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::abi::AbiDefinition;
use crate::debug_locations::resolver::Resolver;
use crate::debug_locations::sources::Sources;
use crate::scope::source_unit::SourceUnitScope;

/// The compilation unit scope: the standard-JSON input's settings every definition of the unit is
/// lowered under.
pub struct CompilationUnitScope<'input> {
    /// The outputs requested for each contract.
    pub output_selection: &'input solx_standard_json::InputSelection,
    /// The EVM version the code targets.
    pub evm_version: EVMVersion,
    /// How revert reasons are kept in the code.
    pub revert_strings: RevertStrings,
    /// The sources the debug locations resolve against.
    pub sources: &'input Sources<'input>,
    /// The input's sources, which the reported errors quote.
    pub input_sources: &'input BTreeMap<String, solx_standard_json::InputSource>,
    /// Whether the output reports the pipeline benchmarks.
    pub report_benchmarks: bool,
}

impl CompilationUnitScope<'_> {
    /// Lowers every definition of `unit` and adds its output to `output` in source order, with the
    /// benchmarks the lowering recorded when they are requested.
    ///
    /// # Errors
    ///
    /// Returns the first error a lowering returned.
    pub fn compilation_unit(
        &self,
        unit: &CompilationUnit,
        output: &mut solx_standard_json::Output,
    ) -> anyhow::Result<()> {
        let lowered = self.members(Self::members_by_cost(unit));
        self.add_outputs(output, lowered)
    }

    /// The members of every source, numbered in source order and sorted costliest first, so the
    /// largest objects start lowering first. A member that deploys nothing costs nothing.
    fn members_by_cost(unit: &CompilationUnit) -> Vec<(usize, SourceUnitMember)> {
        let mut members = unit
            .files()
            .flat_map(|file| file.ast().members().iter().collect::<Vec<_>>())
            .enumerate()
            .collect::<Vec<_>>();
        members.sort_by_cached_key(|(_, member)| {
            Reverse(
                AbiDefinition::from_member(member)
                    .and_then(AbiDefinition::into_object)
                    .map_or(0, |object| object.estimated_lowering_cost()),
            )
        });
        members
    }

    /// Lowers `members` on the global thread pool, or on this thread when LLVM is built without
    /// threads, keyed by their number in source order. A member that declares no definition yields
    /// nothing.
    fn members(&self, members: Vec<(usize, SourceUnitMember)>) -> BTreeMap<usize, Lowered> {
        let lower = |melior: &OnceCell<melior::Context>,
                     (index, member): (usize, SourceUnitMember)| {
            let definition = AbiDefinition::from_member(&member)?;
            let name = definition.name().name().to_owned();
            let contract = self.definition(melior, definition, &name);
            Some((
                index,
                Lowered {
                    member,
                    name,
                    contract,
                },
            ))
        };
        // An LLVM built without threads has locks that do nothing, so only one thread at a time
        // may use LLVM and MLIR.
        if inkwell::support::is_multithreaded() {
            // A melior context serves one thread, so each worker lowers off a context of its own.
            members
                .into_iter()
                .par_bridge()
                .map_init(OnceCell::new, |melior, member| lower(melior, member))
                .flatten()
                .collect()
        } else {
            let melior = OnceCell::new();
            members
                .into_iter()
                .filter_map(|member| lower(&melior, member))
                .collect()
        }
    }

    /// The standard-JSON output of `definition`: its ABI, and the deploy and runtime code of the
    /// object it deploys, lowered off the melior context in `melior`. `None` when Slang cannot
    /// compute its ABI.
    fn definition(
        &self,
        melior: &OnceCell<melior::Context>,
        definition: AbiDefinition,
        name: &str,
    ) -> Option<DefinitionOutput> {
        let abi_value = definition.abi()?.into_value();
        let method_identifiers = definition.method_identifiers();
        let Some(object) = definition.into_object() else {
            let contract = Contract::new_abi(abi_value, method_identifiers);
            return Some(Ok((contract, Vec::new())));
        };
        let storage_layout = object.storage_layout()?;
        let selected = |selector| {
            self.output_selection
                .check_selection(object.file_id().as_str(), Some(name), selector)
        };
        let mut profiler = Profiler::default();
        let contract = SourceUnitScope::object(
            melior,
            &object,
            &storage_layout,
            self.evm_version,
            self.revert_strings,
            selected(solx_standard_json::InputSelector::BytecodeDebugInfo),
            selected(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo),
            selected(solx_standard_json::InputSelector::MLIR),
            self.sources,
            self.report_benchmarks,
            &mut profiler,
        )
        .map(|mlir| {
            // `convert-sol-to-yul` builds the entry-point dispatcher from the same selectors.
            let contract = Contract::new_mlir(mlir, abi_value, method_identifiers);
            let benchmarks = if self.report_benchmarks {
                profiler.to_vec()
            } else {
                Vec::new()
            };
            (contract, benchmarks)
        });
        Some(contract)
    }

    /// Adds each lowered definition's output and benchmarks to `output` in source order, and reports
    /// each definition whose ABI Slang cannot compute.
    ///
    /// # Errors
    ///
    /// Returns the first error a lowering returned.
    fn add_outputs(
        &self,
        output: &mut solx_standard_json::Output,
        lowered: BTreeMap<usize, Lowered>,
    ) -> anyhow::Result<()> {
        for Lowered {
            member,
            name,
            contract,
        } in lowered.into_values()
        {
            match contract {
                Some(contract) => {
                    let (contract, object_benchmarks) = contract?;
                    let file_id = member
                        .calculate_file_id()
                        .expect("slang locates every definition");
                    output
                        .contracts
                        .entry(file_id.to_string())
                        .or_default()
                        .insert(name, contract);
                    output.benchmarks.extend(object_benchmarks);
                }
                None => {
                    let (file_id, text_range) = Resolver::source_range(&member);
                    output.errors.push(OutputError::new_error_with_data(
                        Some(file_id.as_str()),
                        None,
                        format!("Slang cannot compute the ABI of `{name}`."),
                        Some(SourceLocation::new(
                            file_id.to_string(),
                            text_range.start as isize,
                            text_range.end as isize,
                        )),
                        Some(self.input_sources),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// A definition's standard-JSON output and, when they are requested, the benchmarks its lowering
/// recorded.
type DefinitionOutput = anyhow::Result<(Contract, Vec<(String, u64)>)>;

/// What lowering one source-unit member yields.
struct Lowered {
    /// The member declaring the definition, which locates it.
    member: SourceUnitMember,
    /// The definition's name, which keys its output.
    name: String,
    /// The definition's output, or `None` when Slang cannot compute its ABI.
    contract: Option<DefinitionOutput>,
}
