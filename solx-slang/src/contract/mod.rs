//!
//! Contract definition emission to Sol dialect MLIR.
//!

pub mod function;
pub mod state_variable;
pub mod storage_slot;

use std::collections::BTreeMap;
use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;

use solx_mlir::Block;
use solx_mlir::Contract;
use solx_mlir::Function;
use solx_mlir::Type;

use crate::contract::storage_slot::StorageSlot;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Emits a `sol.contract` wrapping a `sol.func` per function and returns the contract's ABI
    /// `method_identifiers` map (externally-dispatchable signature to 4-byte selector, lower-case
    /// hex); `convert-sol-to-yul` builds the entry-point dispatcher from the function selectors.
    /// Function signatures are pre-registered for call resolution before any body is emitted.
    /// Inherited state variables are not yet declared: derived contracts do not compile through
    /// this path.
    pub fn contract_definition(&mut self, node: &ContractDefinition) -> BTreeMap<String, String> {
        let contract_name = node.name().name();

        for function in node.functions().into_iter().chain(node.constructor()) {
            let parameter_types = function
                .parameters()
                .iter()
                .map(|parameter| self.typing(parameter.get_type()))
                .collect();
            let return_types = function
                .returns()
                .map(|returns| {
                    returns
                        .iter()
                        .map(|parameter| self.typing(parameter.get_type()))
                        .collect()
                })
                .unwrap_or_default();
            self.function_signatures.insert(
                function.node_id(),
                Function::new(
                    function
                        .compute_internal_signature()
                        .expect("every emitted function has an internal signature"),
                    parameter_types,
                    return_types,
                ),
            );
        }

        let state_variables = node
            .members()
            .iter()
            .filter_map(|member| match member {
                ContractMember::StateVariableDefinition(state_variable) => {
                    Some(state_variable.clone())
                }
                _ => None,
            })
            .collect();
        let storage_layout = match node.compute_abi() {
            Some(abi) => abi
                .storage_layout()
                .iter()
                .map(|item| (item.node_id(), StorageSlot::from(item)))
                .collect(),
            None => HashMap::new(),
        };

        let sol_contract = Contract::define(
            &contract_name,
            solx_mlir::ContractKind::Contract,
            self,
            Block::from(self.module.body()),
        );
        self.contract(
            Type::contract(self.melior, &contract_name, node.is_payable()),
            sol_contract.body,
            state_variables,
            storage_layout,
            |scope| {
                for state_variable in &scope.state_variables {
                    let Some(slot) = scope.storage_layout.get(&state_variable.node_id()) else {
                        continue;
                    };
                    let element_type = scope.source_unit.resolve(
                        &state_variable
                            .get_type()
                            .expect("binder types every state variable"),
                        None,
                    );
                    sol_contract.declare_state_var(
                        &slot.name,
                        element_type,
                        slot.slot,
                        slot.byte_offset,
                        scope,
                    );
                }
                scope.constructor(node);
                for function in node.functions() {
                    scope.function_definition(&function);
                }
            },
        );

        node.functions()
            .into_iter()
            .filter(|function| {
                matches!(function.kind(), FunctionKind::Regular) && function.is_externally_visible()
            })
            .map(|function| {
                (
                    function
                        .compute_canonical_signature()
                        .expect("an externally visible function has a canonical signature"),
                    format!(
                        "{:08x}",
                        function
                            .compute_selector()
                            .expect("an externally visible function has a selector")
                    ),
                )
            })
            .collect()
    }
}
