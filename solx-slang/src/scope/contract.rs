//!
//! The contract scope: the enclosing source unit scope, the `sol.contract` the members are
//! defined into, and the state-variable data a member resolves against.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::FunctionEntry;
use solx_mlir::Type as MlirType;

use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The contract scope: the enclosing source unit scope, the `sol.contract` the members are
/// defined into, and the state-variable data a member resolves against.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    pub source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The `sol.contract` the members are declared and defined into.
    pub contract: Contract<'context>,
    /// The definition id of the object being emitted.
    pub object_id: NodeId,
    /// The definition ids of the functions the contract defines.
    pub defined_functions: HashSet<NodeId>,
    /// The contract's state variable definitions in declaration order.
    pub state_variables: Vec<StateVariableDefinition>,
    /// The state-variable slots keyed by definition id.
    pub storage_layout: HashMap<NodeId, StorageSlot>,
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract: Contract<'context>,
        object: &Object,
    ) -> Self {
        Self {
            source_unit,
            contract,
            object_id: object.node_id(),
            defined_functions: HashSet::new(),
            state_variables: object.state_variables(),
            storage_layout: object.storage_layout(),
        }
    }

    /// Opens the function scope around `emit`: the entry's declared dispatch, a fresh variable
    /// environment, the declared return types a `return` converts to, and checked arithmetic, with
    /// the MLIR cursor on the entry block for the body's duration.
    pub fn function(
        &mut self,
        entry: FunctionEntry<'context>,
        return_types: Vec<MlirType<'context>>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry.block);
        emit(&mut FunctionScope::new(self, entry.dispatch, return_types));
        self.source_unit.mlir.current_block = enclosing;
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
