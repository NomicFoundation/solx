//!
//! Comparison expression lowering to `sol.cmp`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::InequalityExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an equality expression (`==`, `!=`).
    pub fn emit_equality(
        &self,
        _expression: &EqualityExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("comparison: equality")
    }

    /// Lowers an inequality expression (`<`, `<=`, `>`, `>=`).
    pub fn emit_inequality(
        &self,
        _expression: &InequalityExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("comparison: inequality")
    }
}
