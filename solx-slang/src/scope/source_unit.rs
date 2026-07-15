//!
//! The source unit scope: the owned MLIR context that every nested scope emits into.
//!

use std::collections::HashMap;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Function;
use solx_mlir::Type;

use crate::contract::storage_slot::StorageSlot;
use crate::scope::contract::ContractScope;

/// The source unit scope: the owned MLIR context that every nested scope emits into, and the
/// signatures of the functions lowered within it.
pub struct SourceUnitScope<'context> {
    /// The owned MLIR context, surrendered by the conversion into it.
    pub mlir: Context<'context>,
    /// The function signatures keyed by the AST definition id of each function.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
}

impl<'context> SourceUnitScope<'context> {
    /// Wraps the MLIR context for one source unit's emission.
    pub fn new(mlir: Context<'context>) -> Self {
        Self {
            mlir,
            function_signatures: HashMap::new(),
        }
    }

    /// Opens the contract scope around `emit`: the body an enclosed function is defined into, the
    /// state variables and storage layout it resolves against, with the `this` type installed on
    /// the MLIR context for its duration.
    pub fn contract(
        &mut self,
        contract_type: Type<'context>,
        body: Block<'context>,
        state_variables: Vec<StateVariableDefinition>,
        storage_layout: HashMap<NodeId, StorageSlot>,
        emit: impl FnOnce(&mut ContractScope<'_, 'context>),
    ) {
        self.mlir.current_contract_type = Some(contract_type);
        emit(&mut ContractScope::new(
            self,
            body,
            state_variables,
            storage_layout,
        ));
        self.mlir.current_contract_type = None;
    }

    /// The pre-registered signature of `definition_node_id`'s function.
    pub fn function_signature(&self, definition_node_id: NodeId) -> Function<'context> {
        self.function_signatures
            .get(&definition_node_id)
            .cloned()
            .expect("the contract lowering pre-registers every function")
    }
}

impl<'context> Deref for SourceUnitScope<'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.mlir
    }
}

impl<'context> From<SourceUnitScope<'context>> for Context<'context> {
    fn from(scope: SourceUnitScope<'context>) -> Self {
        scope.mlir
    }
}
