//!
//! Source unit emission: lowering a file's contracts and libraries through the per-file MLIR
//! scope.
//!

use std::cell::OnceCell;
use std::collections::BTreeMap;

use slang_solidity_v2::ast::SourceUnit;

use solx_mlir::Context;
use solx_mlir::DebugInfoRequest;
use solx_standard_json::InputSource;
use solx_standard_json::OutputError;
use solx_standard_json::output::contract::Contract;
use solx_standard_json::output::error::source_location::SourceLocation;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::abi::AbiDefinition;
use crate::contract::object::Object;
use crate::debug_locations::DebugLocations;
use crate::debug_locations::sources::Sources;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Lowers every contract and library the unit deploys into standard-JSON contract outputs
    /// keyed by definition name, each in its own MLIR module off the frontend's melior context. An
    /// abstract contract and an interface deploy nothing and produce no module, only their ABI and
    /// method identifiers.
    ///
    /// A definition whose ABI Slang cannot compute produces no output and pushes an error located
    /// at it to `messages`, with `input_sources` quoting the source.
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
        melior: &OnceCell<melior::Context>,
        unit: &SourceUnit,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        selected: impl Fn(&str, solx_standard_json::InputSelector) -> bool,
        sources: &Sources<'_>,
        pass_timing: bool,
        profiler: &mut Profiler,
        input_sources: &BTreeMap<String, InputSource>,
        messages: &mut Vec<OutputError>,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let mut contracts = BTreeMap::new();
        for member in unit.members().iter() {
            let Some(definition) = AbiDefinition::from_member(&member) else {
                continue;
            };
            let name = definition.name().name().to_owned();
            let Some(abi) = definition.abi() else {
                let (file_id, text_range) = definition.source_range();
                messages.push(OutputError::new_error_with_data(
                    Some(file_id.as_str()),
                    None,
                    format!("Slang cannot compute the ABI of `{name}`."),
                    Some(SourceLocation::new(
                        file_id.to_string(),
                        text_range.start as isize,
                        text_range.end as isize,
                    )),
                    Some(input_sources),
                ));
                continue;
            };
            let abi_value = abi.to_value();
            let method_identifiers = definition.method_identifiers();
            let Some(object) = definition.into_object(abi) else {
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

            let identifier = object.identifier();
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
                        Object::Contract(contract, _) => debug_locations.location(contract),
                        Object::Library(library, _) => debug_locations.location(library),
                    },
                    debug_info,
                    unit.get_file_id().as_str(),
                ),
                debug_locations,
            );
            let run_emission =
                profiler.start_pipeline_element(format!("Compiler_EmitSol:{identifier}").as_str());
            scope.object_definition(&object);
            run_emission.borrow_mut().finish();
            let mlir = Context::from(scope).finalize_module(
                object.deploy_dependencies(),
                object.runtime_dependencies(),
                selected(name.as_str(), solx_standard_json::InputSelector::MLIR),
                pass_timing,
                profiler,
            )?;
            contracts.insert(
                name,
                Contract::new_mlir(mlir, abi_value, method_identifiers),
            );
        }
        Ok(contracts)
    }
}
