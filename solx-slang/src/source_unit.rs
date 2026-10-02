//!
//! Source unit emission: lowering a file's contracts and libraries through the per-file MLIR
//! scope.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;

use solx_mlir::Context;
use solx_mlir::DebugInfoRequest;
use solx_standard_json::output::contract::Contract;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::contract::object::Object;
use crate::debug_locations::DebugLocations;
use crate::debug_locations::sources::Sources;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Lowers every contract and library the unit deploys into standard-JSON contract outputs
    /// keyed by definition name, each in its own MLIR module off the file's melior context. An
    /// abstract contract and an interface deploy nothing and produce no module.
    ///
    /// `selected` tells whether an output selector is requested for a contract, by name. The
    /// MLIR selector captures the Sol dialect text.
    ///
    /// Either debug-info selector switches locations on for the object: the scope resolves the
    /// nodes it lowers through `sources`, and the module is created at the definition's location
    /// with a debug-info compile unit.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    pub fn source_unit(
        unit: &SourceUnit,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        selected: impl Fn(&str, solx_standard_json::InputSelector) -> bool,
        sources: &Sources<'_>,
        profiler: &mut Profiler,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let mut melior = None;
        let mut contracts = BTreeMap::new();
        for member in unit.members().iter() {
            let object = match member {
                SourceUnitMember::ContractDefinition(contract) if !contract.is_abstract() => {
                    Object::Contract(contract.clone())
                }
                SourceUnitMember::LibraryDefinition(library) => Object::Library(library.clone()),
                _ => continue,
            };

            let melior = melior.get_or_insert_with(|| {
                let run_context_creation = profiler.start_pipeline_element(
                    format!("solx_CreateMLIRContext:{}", unit.get_file_id()).as_str(),
                );
                let melior = Context::create_melior_context();
                run_context_creation.borrow_mut().finish();
                melior
            });

            let identifier = object.identifier();
            let name = object.name().name().to_owned();
            let debug_info = DebugInfoRequest {
                deploy: selected(
                    name.as_str(),
                    solx_standard_json::InputSelector::BytecodeDebugInfo,
                ),
                runtime: selected(
                    name.as_str(),
                    solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                ),
            };
            let mut debug_locations =
                DebugLocations::new(melior, debug_info.any().then_some(sources));
            let mut scope = SourceUnitScope::new(
                Context::new(
                    melior,
                    evm_version,
                    revert_strings,
                    match &object {
                        Object::Contract(contract) => debug_locations.location(contract),
                        Object::Library(library) => debug_locations.location(library),
                    },
                    debug_info,
                    unit.get_file_id().as_str(),
                ),
                debug_locations,
            );
            let run_emission =
                profiler.start_pipeline_element(format!("solx_EmitSol:{identifier}").as_str());
            let method_identifiers = scope.object_definition(&object);
            run_emission.borrow_mut().finish();
            let mlir = Context::from(scope).finalize_module(
                identifier.as_str(),
                selected(name.as_str(), solx_standard_json::InputSelector::MLIR),
                profiler,
            )?;
            contracts.insert(name, Contract::new_mlir(mlir, method_identifiers));
        }
        Ok(contracts)
    }
}
