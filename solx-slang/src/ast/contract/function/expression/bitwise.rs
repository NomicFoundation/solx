//!
//! Bitwise and shift expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::ShiftExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a bitwise `&` expression.
    pub fn emit_bitwise_and(
        &self,
        _expression: &BitwiseAndExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("bitwise: and")
    }

    /// Lowers a bitwise `|` expression.
    pub fn emit_bitwise_or(
        &self,
        _expression: &BitwiseOrExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("bitwise: or")
    }

    /// Lowers a bitwise `^` expression.
    pub fn emit_bitwise_xor(
        &self,
        _expression: &BitwiseXorExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("bitwise: xor")
    }

    /// Lowers a shift expression (`<<`, `>>`).
    pub fn emit_shift(
        &self,
        _expression: &ShiftExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("bitwise: shift")
    }
}
