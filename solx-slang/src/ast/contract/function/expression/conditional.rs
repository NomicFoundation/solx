//!
//! Conditional (ternary `?:`) expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ConditionalExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a ternary `cond ? a : b` expression.
    pub fn emit_conditional(
        &self,
        _conditional: &ConditionalExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("conditional expression")
    }
}
