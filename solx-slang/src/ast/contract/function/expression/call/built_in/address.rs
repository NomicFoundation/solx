//!
//! Address value-transfer member built-ins: `address.send(amount)` and
//! `address.transfer(amount)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `address.send(amount)` to `sol.send`, yielding the `bool` success
    /// status without reverting on failure.
    pub fn emit_address_send(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (address, amount, block) =
            self.emit_value_transfer_operands(access, arguments, block)?;
        let status = self
            .expression_emitter
            .state
            .builder
            .emit_sol_send(address, amount, &block);
        Ok((Some(status), block))
    }

    /// Lowers `address.transfer(amount)` to `sol.transfer`, which reverts on
    /// failure.
    pub fn emit_address_transfer(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (address, amount, block) =
            self.emit_value_transfer_operands(access, arguments, block)?;
        self.expression_emitter
            .state
            .builder
            .emit_sol_transfer(address, amount, &block);
        Ok((None, block))
    }

    /// Evaluates the recipient address and the wei amount, widening the amount
    /// to `ui256` (a narrow literal such as `transfer(1)` keeps its source
    /// type). Shared by `send` and `transfer`.
    fn emit_value_transfer_operands(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (address, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let amount_expression = arguments
            .iter()
            .next()
            .expect("send/transfer takes one wei amount");
        let (amount, block) = self
            .expression_emitter
            .emit_value(&amount_expression, block)?;
        let builder = &self.expression_emitter.state.builder;
        let amount = builder.emit_sol_cast(amount, builder.types.ui256, &block);
        Ok((address, amount, block))
    }
}
