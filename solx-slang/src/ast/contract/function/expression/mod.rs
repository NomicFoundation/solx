//!
//! Expression lowering to MLIR SSA values.
//!

/// Function and built-in call lowering.
pub mod call;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+); `false` inside `unchecked {}` blocks.
    pub checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        checked: bool,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            checked,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns `None` for void expressions (calls with no return value); use
    /// [`Self::emit_value`] when a value is required.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs.
    pub fn emit(
        &self,
        expression: &Expression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!(
            "expression lowering: {:?}",
            std::mem::discriminant(expression)
        )
    }

    /// Emits MLIR for an expression that must produce a value.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression is void or unsupported.
    pub fn emit_value(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(expression, block)?;
        let value = value.ok_or_else(|| anyhow::anyhow!("expression produced no value"))?;
        Ok((value, block))
    }

    /// Emits the contract's state-variable initializers into `block`, returning
    /// the continuation block.
    ///
    /// # Errors
    ///
    /// Returns an error if an initializer contains unsupported constructs.
    pub fn emit_state_var_initializers(
        &self,
        contract: &ContractDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        for member in contract.members().iter() {
            if let ContractMember::StateVariableDefinition(variable) = member
                && variable.value().is_some()
            {
                unimplemented!("state variable initializer lowering");
            }
        }
        Ok(block)
    }
}
