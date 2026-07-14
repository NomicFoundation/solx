//!
//! Contract definition emission to Sol dialect MLIR.
//!

pub mod function;
pub mod state_variable;
pub mod storage_slot;

use std::collections::BTreeMap;
use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition as SlangContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression as SlangExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition as SlangStateVariableDefinition;

use solx_mlir::Block;
use solx_mlir::Contract;
use solx_mlir::Type;

use crate::contract::function::FunctionDefinition;
use crate::contract::function::expression::Expression;
use crate::contract::state_variable::StateVariableDefinition;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::FunctionScope;
use crate::scope::SourceUnitScope;

codegen!(
    ContractDefinition {
        /// Emits a `sol.contract` wrapping a `sol.func` per function; `convert-sol-to-yul` builds
        /// the entry-point dispatcher from the function selectors. Function signatures are
        /// pre-registered for call resolution before any body is emitted. Inherited state
        /// variables are not yet declared: derived contracts do not compile through this path.
        pub fn emit(node: &SlangContractDefinition, scope: &mut SourceUnitScope) {
            let contract_name = node.name().name();

            for function in node.functions().into_iter().chain(node.constructor()) {
                let (parameter_types, return_types) =
                    FunctionDefinition::signature_types(&function, scope);
                scope.register_function_signature(
                    function.node_id(),
                    function
                        .compute_internal_signature()
                        .expect("every emitted function has an internal signature"),
                    parameter_types,
                    return_types,
                );
            }

            let sol_contract = Contract::define(
                &contract_name,
                solx_mlir::ContractKind::Contract,
                scope,
                Block::from(scope.module.body()),
            );
            scope.contract(
                Self::storage_layout(node),
                Type::contract(scope.melior, &contract_name, node.is_payable()),
                sol_contract.body,
                |scope| {
                    for state_variable in Self::state_variables(node) {
                        let Some(slot) = scope.storage_layout().get(&state_variable.node_id())
                        else {
                            continue;
                        };
                        let element_type =
                            codegen!(@result_type StateVariableDefinition, state_variable, scope);
                        sol_contract.declare_state_var(
                            &slot.name,
                            element_type,
                            slot.slot,
                            slot.byte_offset,
                            scope,
                        );
                    }
                    FunctionDefinition::emit_constructor(node, scope);
                    for function in node.functions() {
                        FunctionDefinition::emit(&function, node, scope);
                    }
                },
            );
        }

        /// Emits every state variable's inline initializer (`T x = <expr>;`) in source order as
        /// the constructor prologue, storing each into its storage slot. Reference-typed slots
        /// take a `sol.copy`; value-typed slots coerce to the declared element type and
        /// `sol.store`.
        pub fn emit_initializers(contract: &SlangContractDefinition, scope: &mut FunctionScope) {
            for state_variable in Self::state_variables(contract) {
                let Some(slot_name) = scope
                    .contract()
                    .storage_layout()
                    .get(&state_variable.node_id())
                    .map(|slot| slot.name.clone())
                else {
                    continue;
                };
                let Some(initializer) = state_variable.value() else {
                    continue;
                };
                if matches!(initializer, SlangExpression::ArrayExpression(_)) {
                    unimplemented!(
                        "array-literal state variable initializers are not yet supported"
                    );
                }
                let (storage_ref, element_type) =
                    StateVariableDefinition::storage_place(&state_variable, &slot_name, scope);
                let value = Expression::emit(&initializer, scope);
                if storage_ref.r#type() == element_type {
                    storage_ref.copy_from(value, scope);
                } else {
                    storage_ref.store(value.coerce(element_type, scope), scope);
                }
            }
        }

        /// The storage layout re-keyed from Slang's ABI, mapping each state variable's node ID to
        /// its slot index and byte offset. Empty when the ABI is unavailable.
        pub fn storage_layout(contract: &SlangContractDefinition) -> HashMap<NodeId, StorageSlot> {
            let Some(abi) = contract.compute_abi() else {
                return HashMap::new();
            };
            abi.storage_layout()
                .iter()
                .map(|item| {
                    (
                        item.node_id(),
                        StorageSlot {
                            slot: item.slot(),
                            byte_offset: item.offset() as u32,
                            name: format!("{}_{}", item.label(), item.node_id()),
                        },
                    )
                })
                .collect()
        }

        /// The ABI `method_identifiers` map: externally-dispatchable signature to 4-byte
        /// selector, lower-case hex.
        pub fn method_identifiers(contract: &SlangContractDefinition) -> BTreeMap<String, String> {
            contract
                .members()
                .iter()
                .filter_map(|member| {
                    let ContractMember::FunctionDefinition(function) = member else {
                        return None;
                    };
                    let signature = function.compute_canonical_signature()?;
                    let selector = function.compute_selector()?;
                    Some((signature, format!("{selector:08x}")))
                })
                .collect()
        }

        /// The state variable definitions among the contract's members, in declaration order.
        pub fn state_variables(
            contract: &SlangContractDefinition,
        ) -> Vec<SlangStateVariableDefinition> {
            contract
                .members()
                .iter()
                .filter_map(|member| match member {
                    ContractMember::StateVariableDefinition(state_variable) => {
                        Some(state_variable.clone())
                    }
                    _ => None,
                })
                .collect()
        }
    }
);
