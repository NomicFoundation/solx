//!
//! Short-circuit logical expression lowering (`&&`, `||`, `!`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a short-circuiting `&&` expression.
    pub fn emit_and(
        &self,
        _left: &Expression,
        _right: &Expression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("logical: and")
    }

    /// Lowers a short-circuiting `||` expression.
    pub fn emit_or(
        &self,
        _left: &Expression,
        _right: &Expression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("logical: or")
    }
}
