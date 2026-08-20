//!
//! Contract and library definition emission to Sol dialect MLIR.
//!

pub mod function;
pub mod getter;
pub mod object;
pub mod state_variable;
pub mod storage_slot;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Block;
use solx_mlir::Contract;
use solx_mlir::Type as MlirType;

use crate::contract::object::Object;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Emits `object`'s `sol.contract` and returns its ABI `method_identifiers` map.
    pub fn object_definition(&mut self, object: &Object) -> BTreeMap<String, String> {
        let identifier = object.identifier();
        let contract = Contract::define(
            identifier.as_str(),
            object.kind(),
            self,
            Block::from(self.module.body()),
        );
        self.contract(
            MlirType::contract(self.melior, identifier.as_str(), object.is_payable()),
            contract,
            object,
            |scope| scope.members(object),
        );

        object.method_identifiers()
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the object's members: the state-variable declarations, the contract's constructor,
    /// the functions, and the getters.
    fn members(&mut self, object: &Object) {
        for state_variable in self.state_variables.iter() {
            match state_variable.attributes().mutability() {
                StateVariableMutability::Mutable | StateVariableMutability::Transient => {
                    let slot = self
                        .storage_layout
                        .get(&state_variable.node_id())
                        .expect("slang lays out every state variable");
                    let element_type = self.source_unit.resolve(
                        &state_variable
                            .get_type()
                            .expect("binder types every state variable"),
                        None,
                    );
                    self.contract.declare_state_var(
                        &SourceUnitScope::state_variable_symbol(state_variable),
                        element_type,
                        slot.slot,
                        slot.byte_offset,
                        matches!(
                            state_variable.attributes().mutability(),
                            StateVariableMutability::Transient
                        ),
                        self,
                    );
                }
                StateVariableMutability::Immutable => {
                    let element_type = self.source_unit.resolve(
                        &state_variable
                            .get_type()
                            .expect("binder types every state variable"),
                        None,
                    );
                    self.contract.declare_immutable(
                        &SourceUnitScope::state_variable_symbol(state_variable),
                        element_type,
                        self,
                    );
                }
                StateVariableMutability::Constant => {}
            }
        }
        if let Object::Contract(node) = object {
            self.constructor(node);
        }
        for function in object.functions().iter() {
            self.function_definition(function);
        }
        for state_variable in object.public_state_variables() {
            self.state_variable_getter(&state_variable);
        }
    }
}
