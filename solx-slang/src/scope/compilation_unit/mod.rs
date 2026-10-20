//!
//! The compilation unit scope: the standard-JSON input's settings every definition of the unit is
//! lowered under. The definitions lower on the global thread pool, the costliest first.
//!

pub mod definition_output;

use std::cell::OnceCell;
use std::cmp::Reverse;
use std::collections::BTreeMap;

use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationUnit;

use solx_standard_json::output::contract::Contract;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::abi::AbiDefinition;
use crate::error::Error;
use crate::scope::source_unit::SourceUnitScope;

use self::definition_output::DefinitionOutput;

/// The compilation unit scope: the standard-JSON input's settings every definition of the unit is
/// lowered under.
pub struct CompilationUnitScope<'input> {
    /// The outputs requested for each contract.
    pub output_selection: &'input solx_standard_json::InputSelection,
    /// The EVM version the code targets.
    pub evm_version: EVMVersion,
    /// How revert reasons are kept in the code.
    pub revert_strings: RevertStrings,
    /// The input's sources, which the debug locations resolve against and the reported errors
    /// quote.
    pub input_sources: &'input BTreeMap<String, solx_standard_json::InputSource>,
    /// Whether the output reports the pipeline benchmarks.
    pub report_benchmarks: bool,
    /// Whether the selection requests an output only codegen produces.
    pub run_codegen: bool,
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
        for definition_output in self.members(Self::members_by_cost(unit))?.into_values() {
            definition_output.write_to_standard_json(output, self.input_sources);
        }
        Ok(())
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
    ///
    /// # Errors
    ///
    /// Returns the first error a lowering returned.
    fn members(
        &self,
        members: Vec<(usize, SourceUnitMember)>,
    ) -> anyhow::Result<BTreeMap<usize, DefinitionOutput>> {
        let lower = |melior: &OnceCell<melior::Context>,
                     (index, member): (usize, SourceUnitMember)| {
            let definition = AbiDefinition::from_member(&member)?;
            Some(
                self.definition(melior, definition, member)
                    .map(|output| (index, output)),
            )
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

    /// The output of `definition`, which `member` declares, lowered off the melior context in
    /// `melior`.
    ///
    /// # Errors
    ///
    /// Returns the error of a lowering that failed.
    fn definition(
        &self,
        melior: &OnceCell<melior::Context>,
        definition: AbiDefinition,
        member: SourceUnitMember,
    ) -> anyhow::Result<DefinitionOutput> {
        let name = definition.name().name().to_owned();
        let mut profiler = Profiler::default();
        let contract = match self.contract(melior, definition, &name, &mut profiler) {
            Ok(contract) => Ok(contract?),
            Err(error) => Err(error),
        };
        let benchmarks = if self.report_benchmarks {
            profiler.to_vec()
        } else {
            Vec::new()
        };
        Ok(DefinitionOutput::new(member, name, contract, benchmarks))
    }

    /// The standard-JSON contract of `definition`: its ABI, the storage layouts the selection
    /// requests, and the deploy and runtime code of the object it deploys, lowered off the melior
    /// context in `melior` and timed by `profiler` when the selection requests an output only
    /// codegen produces. `Err` is the first output Slang cannot compute, and the inner error a
    /// lowering that failed.
    fn contract(
        &self,
        melior: &OnceCell<melior::Context>,
        definition: AbiDefinition,
        name: &str,
        profiler: &mut Profiler,
    ) -> Result<anyhow::Result<Contract>, Error> {
        let file_id = definition.file_id().clone();
        let selected = |selector| {
            self.output_selection
                .check_selection(file_id.as_str(), Some(name), selector)
        };
        let abi = definition.abi()?;
        let method_identifiers = definition.method_identifiers();
        let storage_layout = selected(solx_standard_json::InputSelector::StorageLayout)
            .then(|| definition.storage_layout())
            .transpose()?;
        let transient_storage_layout =
            selected(solx_standard_json::InputSelector::TransientStorageLayout)
                .then(|| definition.transient_storage_layout())
                .transpose()?;
        let Some(object) = definition.into_object().filter(|_| self.run_codegen) else {
            return Ok(Ok(Contract::new_abi(
                abi,
                method_identifiers,
                storage_layout,
                transient_storage_layout,
            )));
        };
        let storage_slots = object.storage_slots()?;
        let contract = SourceUnitScope::object(
            melior,
            &object,
            &storage_slots,
            self.evm_version,
            self.revert_strings,
            selected(solx_standard_json::InputSelector::BytecodeDebugInfo),
            selected(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo),
            selected(solx_standard_json::InputSelector::MLIR),
            self.input_sources,
            self.report_benchmarks,
            profiler,
        )
        .map(|mlir| {
            // `convert-sol-to-yul` builds the entry-point dispatcher from the same selectors.
            Contract::new_mlir(
                mlir,
                abi,
                method_identifiers,
                storage_layout,
                transient_storage_layout,
            )
        });
        Ok(contract)
    }
}
