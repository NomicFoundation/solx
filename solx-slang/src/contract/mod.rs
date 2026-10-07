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

use std::collections::HashMap;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Block;
use solx_mlir::Contract;
use solx_mlir::Type as MlirType;
use solx_utils::CodeSegment;

use crate::contract::indirect_callees::IndirectCallees;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Emits the `sol.contract` of the segment `indirect_callees` are seen from over
    /// `storage_layout`. The location cursor starts on the definition; a member with a node of its
    /// own narrows it, and the synthesized constructor keeps it.
    pub fn object_definition(
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
