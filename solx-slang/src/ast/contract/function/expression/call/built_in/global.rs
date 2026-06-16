//!
//! Address value-transfer member calls: `address.send`/`transfer`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits `address.send(value)` as `sol.send`, yielding the success flag.
    pub fn emit_address_send(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let builder = &self.state.builder;
        let BlockAnd { value: addr, block } = access.operand().emit(self, block);
        let BlockAnd {
            value: values,
            block,
        } = arguments.emit(self, block);
        // `sol.send` takes a `ui256` amount; a narrow literal (`r.send(0)` → ui8)
        // must be widened first, like `address.transfer`.
        let amount = AstValue::from(values[0])
            .cast(
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .into_mlir();
        let value = sol_op!(
            builder,
            block,
            SendOperation
                .addr(addr)
                .val(amount)
                .status(AstType::signless(
                    builder.context,
                    solx_utils::BIT_LENGTH_BOOLEAN
                ))
        );
        (Some(value), block)
    }

    /// Emits `address.transfer(value)` as `sol.transfer` (no result value).
    pub fn emit_address_transfer(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let builder = &self.state.builder;
        let BlockAnd { value: addr, block } = access.operand().emit(self, block);
        let BlockAnd {
            value: values,
            block,
        } = arguments.emit(self, block);
        // `sol.transfer` takes a `ui256` amount; a narrow literal (`x.transfer(1)`
        // → ui8) must be widened first.
        let amount = AstValue::from(values[0])
            .cast(
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .into_mlir();
        sol_op_void!(builder, block, TransferOperation.addr(addr).val(amount));
        (None, block)
    }
}
