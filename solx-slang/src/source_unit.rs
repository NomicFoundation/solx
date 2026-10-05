//!
//! Source unit emission: lowering a file's contracts and libraries through the per-file MLIR
//! scope.
//!

use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::collections::HashMap;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;

use solx_mlir::Context;
use solx_mlir::MlirOutput;
use solx_mlir::SegmentOutput;
use solx_standard_json::output::contract::Contract;
use solx_utils::CodeSegment;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::abi::AbiDefinition;
use crate::contract::indirect_callees::IndirectCallees;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::debug_locations::DebugLocations;
use crate::debug_locations::sources::Sources;
use crate::scope::source_unit::SourceUnitScope;

/// The definitions of a unit whose ABI Slang cannot compute, each by name and the member declaring
/// it.
pub type UncomputableAbis = Vec<(String, SourceUnitMember)>;

impl<'context> SourceUnitScope<'context> {
    /// Lowers every contract and library the unit deploys into standard-JSON contract outputs
    /// keyed by definition name, each segment in its own MLIR module off the frontend's melior
    /// context: the deploy segment first, whose indirect callees the runtime segment takes. An
    /// abstract contract and an interface deploy nothing and produce no module, only their ABI and
    /// method identifiers.
    /// The storage layout is computed once for both segments.
    ///
    /// A definition whose ABI Slang cannot compute produces no output and is returned by name
    /// next to the outputs, as the member declaring it.
    ///
    /// `selected` tells whether an output selector is requested for a contract, by name. The
    /// MLIR selector captures the Sol dialect text.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    pub fn source_unit(
        melior: &OnceCell<melior::Context>,
        unit: &SourceUnit,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        selected: impl Fn(&str, solx_standard_json::InputSelector) -> bool,
        sources: &Sources<'_>,
        pass_timing: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<(BTreeMap<String, Contract>, UncomputableAbis)> {
        let mut contracts = BTreeMap::new();
        let mut uncomputable_abis = UncomputableAbis::new();
        for member in unit.members().iter() {
            let Some(definition) = AbiDefinition::from_member(&member) else {
                continue;
            };
            let name = definition.name().name().to_owned();
            let Some(abi) = definition.abi() else {
                uncomputable_abis.push((name, member));
                continue;
            };
            let storage_layout = abi.storage_layout();
            let abi_value = abi.into_value();
            let method_identifiers = definition.method_identifiers();
            let Some(object) = definition.into_object() else {
                contracts.insert(name, Contract::new_abi(abi_value, method_identifiers));
                continue;
            };

            let melior = melior.get_or_init(|| {
                let run_context_creation =
                    profiler.start_pipeline_element("Compiler_CreateMLIRContext");
                let melior = Context::create_melior_context();
                run_context_creation.borrow_mut().finish();
                melior
            });

            let capture_sol = selected(name.as_str(), solx_standard_json::InputSelector::MLIR);
            let mut deploy_indirect_callees = BTreeMap::new();
            let deploy = SourceUnitScope::object_segment(
                melior,
                unit,
                &object,
                &storage_layout,
                IndirectCallees::Taken(&mut deploy_indirect_callees),
                evm_version,
                revert_strings,
                selected(
                    name.as_str(),
                    solx_standard_json::InputSelector::BytecodeDebugInfo,
                ),
                capture_sol,
                sources,
                pass_timing,
                profiler,
            )?;
            let runtime = SourceUnitScope::object_segment(
                melior,
                unit,
                &object,
                &storage_layout,
                IndirectCallees::Reachable(&deploy_indirect_callees),
                evm_version,
                revert_strings,
                selected(
                    name.as_str(),
                    solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                ),
                capture_sol,
                sources,
                pass_timing,
                profiler,
            )?;
            // `convert-sol-to-yul` builds the entry-point dispatcher from the same selectors.
            contracts.insert(
                name,
                Contract::new_mlir(
                    MlirOutput::new(deploy, runtime),
                    abi_value,
                    method_identifiers,
                ),
            );
        }
        Ok((contracts, uncomputable_abis))
    }

    /// Lowers the segment of `object` that `indirect_callees` are seen from into a module of its
    /// own and finalizes it.
    ///
    /// With `emit_debug_info`, the scope resolves the nodes it lowers through `sources`, and the
    /// module is created at the definition's location with a debug-info compile unit.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    fn object_segment(
        melior: &'context melior::Context,
        unit: &SourceUnit,
        object: &Object,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        indirect_callees: IndirectCallees<'_>,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        emit_debug_info: bool,
        capture_sol: bool,
        sources: &Sources<'_>,
        pass_timing: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<SegmentOutput> {
        let dependencies = match indirect_callees.segment() {
            CodeSegment::Deploy => object.deploy_dependencies(),
            CodeSegment::Runtime => object.runtime_dependencies(),
        };
        let code_identifier = dependencies.identifier.clone();

        let mut debug_locations = DebugLocations::new(melior, emit_debug_info.then_some(sources));
        let mut scope = SourceUnitScope::new(
            Context::new(
                melior,
                evm_version,
                revert_strings,
                match object {
                    Object::Contract(contract) => debug_locations.location(contract),
                    Object::Library(library) => debug_locations.location(library),
                },
                emit_debug_info,
                unit.get_file_id().as_str(),
            ),
            debug_locations,
        );
        let run_emission =
            profiler.start_pipeline_element(format!("Compiler_EmitSol:{code_identifier}").as_str());
        scope.object_definition(object, storage_layout, indirect_callees);
        run_emission.borrow_mut().finish();
        Context::from(scope).finalize_module(dependencies, capture_sol, pass_timing, profiler)
    }
}
