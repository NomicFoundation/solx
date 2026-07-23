//!
//! The contract scope: the enclosing source unit scope, the block the contract's functions are
//! defined into, and the state-variable data a member resolves against.
//!

use std::collections::HashMap;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Type as MlirType;

use crate::contract::storage_slot::StorageSlot;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The contract scope: the enclosing source unit scope, the block the contract's functions are
/// defined into, and the state-variable data a member resolves against.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    pub source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The block the contract's `sol.func`s are defined into.
    pub contract_body: Block<'context>,
    /// The contract's state variable definitions in declaration order.
    pub state_variables: Vec<StateVariableDefinition>,
    /// The state-variable slots keyed by definition id.
    pub storage_layout: HashMap<NodeId, StorageSlot>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract_body: Block<'context>,
        state_variables: Vec<StateVariableDefinition>,
        storage_layout: HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            source_unit,
            contract_body,
            state_variables,
            storage_layout,
        }
    }

    /// Opens the function scope around `emit`: a fresh variable environment, the declared return
    /// types a `return` converts to, and checked arithmetic, with the MLIR cursor on `entry` for the
    /// body's duration.
    pub fn function(
        &mut self,
        entry: Block<'context>,
        return_types: Vec<MlirType<'context>>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry);
        emit(&mut FunctionScope::new(self, return_types));
        self.source_unit.mlir.current_block = enclosing;
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
