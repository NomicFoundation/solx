//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an assignment expression (`=` and compound `+=`, `-=`, …).
    pub fn emit_assignment(
        &self,
        _assignment: &AssignmentExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("assignment")
    }
}
