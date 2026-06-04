//!
//! Binary arithmetic expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PrefixExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an additive expression (`+`, `-`).
    pub fn emit_additive(
        &self,
        _expression: &AdditiveExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("arithmetic: additive")
    }

    /// Lowers a multiplicative expression (`*`, `/`, `%`).
    pub fn emit_multiplicative(
        &self,
        _expression: &MultiplicativeExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("arithmetic: multiplicative")
    }

    /// Lowers an exponentiation expression (`**`).
    pub fn emit_exponentiation(
        &self,
        _expression: &ExponentiationExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("arithmetic: exponentiation")
    }

    /// Lowers a postfix expression (`x++`, `x--`).
    pub fn emit_postfix(
        &self,
        _expression: &PostfixExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("arithmetic: postfix")
    }

    /// Lowers a prefix expression (`++x`, `--x`, `!x`, `~x`, `-x`).
    pub fn emit_prefix(
        &self,
        _expression: &PrefixExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("arithmetic: prefix")
    }
}
