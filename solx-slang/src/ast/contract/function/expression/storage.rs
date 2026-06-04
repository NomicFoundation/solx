//!
//! State variable storage access: reads, writes, and inline initializers.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits the contract's state-variable initializers into `block`, returning
    /// the continuation block. Contracts without initializers emit nothing.
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
