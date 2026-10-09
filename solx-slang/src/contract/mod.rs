//!
//! Contract and library definition emission to Sol dialect MLIR.
//!

pub mod constructor;
pub mod function;
pub mod getter;
pub mod indirect_callees;
pub mod object;
pub mod state_variable;
pub mod storage_slot;

use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::collections::HashMap;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::MlirOutput;
use solx_mlir::SegmentOutput;
use solx_mlir::Type as MlirType;
use solx_utils::CodeSegment;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::RevertStrings;

use crate::contract::indirect_callees::IndirectCallees;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::debug_locations::DebugLocations;
use crate::debug_locations::sources::Sources;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Lowers `object` over `storage_layout` off the melior context in `melior`, created on first
    /// use, each segment into a module of its own: the deploy segment first, whose indirect
    /// callees the runtime segment takes.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    pub fn object(
        melior: &OnceCell<melior::Context>,
        object: &Object,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        emit_deploy_debug_info: bool,
        emit_runtime_debug_info: bool,
        capture_mlir: bool,
        sources: &Sources<'_>,
        pass_timing: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<MlirOutput> {
        let melior = melior.get_or_init(|| {
            let run_context_creation =
                profiler.start_pipeline_element("Compiler_CreateMLIRContext");
            let melior = Context::create_melior_context();
            run_context_creation.borrow_mut().finish();
            melior
        });

        let mut deploy_indirect_callees = BTreeMap::new();
        let deploy = SourceUnitScope::object_segment(
            melior,
            object,
            storage_layout,
            IndirectCallees::Taken(&mut deploy_indirect_callees),
            evm_version,
            revert_strings,
            emit_deploy_debug_info,
            capture_mlir,
            sources,
            pass_timing,
            profiler,
        )?;
        let runtime = SourceUnitScope::object_segment(
            melior,
            object,
            storage_layout,
            IndirectCallees::Reachable(&deploy_indirect_callees),
            evm_version,
            revert_strings,
            emit_runtime_debug_info,
            capture_mlir,
            sources,
            pass_timing,
            profiler,
        )?;
        Ok(MlirOutput::new(deploy, runtime))
    }

    /// Emits the `sol.contract` of the segment `indirect_callees` are seen from over
    /// `storage_layout`. The location cursor starts on the definition; a member with a node of its
    /// own narrows it, and the synthesized constructor keeps it.
    fn object_definition(
        &mut self,
        object: &Object,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        indirect_callees: IndirectCallees<'_>,
    ) {
        let identifier = object.identifier();
        self.contract(
            MlirType::contract(self.melior, identifier.as_str(), object.is_payable()),
            Contract::define(
                identifier.as_str(),
                object.kind(),
                indirect_callees.segment(),
                self,
                Block::from(self.module.body()),
            ),
            object,
            storage_layout,
            indirect_callees,
            |scope| scope.members(),
        );
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
        object: &Object,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        indirect_callees: IndirectCallees<'_>,
        evm_version: EVMVersion,
        revert_strings: RevertStrings,
        emit_debug_info: bool,
        capture_mlir: bool,
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
                object.file_id().as_str(),
            ),
            debug_locations,
        );
        let run_emission =
            profiler.start_pipeline_element(format!("Compiler_EmitSol:{code_identifier}").as_str());
        scope.object_definition(object, storage_layout, indirect_callees);
        run_emission.borrow_mut().finish();
        Context::from(scope).finalize_module(dependencies, capture_mlir, pass_timing, profiler)
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the segment's members: the state-variable declarations, then the deploy code's
    /// constructor, or the runtime code's entry points, the getters of the public state variables
    /// and the indirect callees. Each state-variable declaration carries its own location.
    fn members(&mut self) {
        let mut immutable_index = 0;
        for state_variable in self.object.state_variables().iter() {
            match state_variable.attributes().mutability() {
                StateVariableMutability::Mutable | StateVariableMutability::Transient => self
                    .at_node(state_variable, |scope| {
                        let slot = scope
                            .storage_layout
                            .get(&state_variable.node_id())
                            .expect("slang lays out every state variable");
                        let element_type = scope.source_unit.resolve(
                            &state_variable
                                .get_type()
                                .expect("binder types every state variable"),
                            None,
                        );
                        scope.contract.declare_state_var(
                            &SourceUnitScope::state_variable_symbol(state_variable),
                            element_type,
                            slot.slot,
                            slot.byte_offset,
                            matches!(
                                state_variable.attributes().mutability(),
                                StateVariableMutability::Transient
                            ),
                            scope,
                        );
                    }),
                StateVariableMutability::Immutable => self.at_node(state_variable, |scope| {
                    let element_type = scope.source_unit.resolve(
                        &state_variable
                            .get_type()
                            .expect("binder types every state variable"),
                        None,
                    );
                    scope.contract.declare_immutable(
                        &SourceUnitScope::state_variable_symbol(state_variable),
                        element_type,
                        immutable_index,
                        scope,
                    );
                    immutable_index += 1;
                }),
                StateVariableMutability::Constant => {}
            }
        }
        match self.segment {
            CodeSegment::Deploy => self.constructor(),
            CodeSegment::Runtime => {
                for function in self.object.entry_points() {
                    self.function_definition(&function);
                }
                for state_variable in self
                    .object
                    .state_variables()
                    .into_iter()
                    .filter(|state_variable| state_variable.is_externally_visible())
                {
                    self.state_variable_getter(&state_variable);
                }
                self.define_indirect_callees();
            }
        }
    }
}
