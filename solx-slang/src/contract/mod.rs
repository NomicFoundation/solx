//!
//! Contract and library definition emission to Sol dialect MLIR.
//!

pub mod constructor_chain;
pub mod function;
pub mod getter;
pub mod object;
pub mod state_variable;
pub mod storage_slot;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Block;
use solx_mlir::Contract;
use solx_mlir::Type as MlirType;

use crate::contract::object::Object;
use crate::scope::contract::ContractScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Emits `object`'s `sol.contract` and returns its ABI `method_identifiers` map.
    pub fn object_definition(&mut self, object: Object) -> BTreeMap<String, String> {
        let identifier = object.identifier();
        let contract = Contract::define(
            identifier.as_str(),
            object.kind(),
            self,
            Block::from(self.module.body()),
        );
        let contract_type =
            MlirType::contract(self.melior, identifier.as_str(), object.is_payable());
        self.contract(contract_type, contract, object, |scope| {
            scope.members();
            scope.method_identifiers()
        })
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Emits the object's members: the state-variable declarations, the constructor, the
    /// functions of its resolved hierarchy, and the getters of its public state variables.
    fn members(&mut self) {
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
        self.constructor();
        let functions = self.functions.clone();
        for function in &functions {
            self.function_definition(function);
        }
        let public_state_variables: Vec<StateVariableDefinition> =
            self.public_state_variables().cloned().collect();
        for state_variable in &public_state_variables {
            self.state_variable_getter(state_variable);
        }
    }

    /// The ABI `method_identifiers` map (externally dispatchable signature to 4-byte selector,
    /// lower-case hex): each function keyed by the signature its selector hashes, each public
    /// state variable by its canonical one. `convert-sol-to-yul` builds the entry-point
    /// dispatcher from the function selectors.
    fn method_identifiers(&self) -> BTreeMap<String, String> {
        self.functions
            .iter()
            .filter(|function| {
                matches!(function.kind(), FunctionKind::Regular) && function.is_externally_visible()
            })
            .map(|function| {
                (
                    function
                        .compute_selector_signature()
                        .expect("an externally visible function has a selector signature"),
                    function
                        .compute_selector()
                        .expect("an externally visible function has a selector"),
                )
            })
            .chain(self.public_state_variables().map(|state_variable| {
                (
                    state_variable
                        .compute_canonical_signature()
                        .expect("a public state variable has a canonical signature"),
                    state_variable
                        .compute_selector()
                        .expect("a public state variable has a selector"),
                )
            }))
            .map(|(signature, selector)| (signature, format!("{selector:08x}")))
            .collect()
    }

    /// The state variables a getter dispatches to.
    fn public_state_variables(&self) -> impl Iterator<Item = &StateVariableDefinition> {
        self.state_variables
            .iter()
            .filter(|state_variable| state_variable.is_externally_visible())
    }
}
