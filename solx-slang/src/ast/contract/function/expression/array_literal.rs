//!
//! Inline array literal expression lowering: `[a, b, c]`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an inline array literal `[a, b, c]`.
    pub fn emit_array_literal(
        &self,
        _array_expression: &ArrayExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("array literal")
    }
}
