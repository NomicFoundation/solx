//!
//! Literal and primary-keyword expression emission: number / boolean / string
//! literals and the `this` keyword.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::FalseKeyword;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::ThisKeyword;
use slang_solidity_v2::ast::TrueKeyword;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ThisOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(DecimalNumberExpression; |node, context, block| {
    let value = node.integer_value().expect(
        "decimal literal must evaluate to an integer after applying any units",
    );
    let result_type = context
        .resolve_slang_type(node.get_type())
        .expect("binder types every decimal literal node");
    let value = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        context.state,
        &block,
    )
    .into_mlir();
    BlockAnd { block, value }
});

expression_emit!(HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("hex literals always evaluate to integers");
    let result_type = context
        .resolve_slang_type(node.get_type())
        .expect("binder types every hex literal node");
    let value = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        context.state,
        &block,
    )
    .into_mlir();
    BlockAnd { block, value }
});

expression_emit!(TrueKeyword; |context, block| {
    let value = AstValue::boolean(true, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

expression_emit!(FalseKeyword; |context, block| {
    let value = AstValue::boolean(false, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

expression_emit!(ThisKeyword; |context, block| {
    let contract_type = context
        .state
        .current_contract_type
        .expect("sol.this emitted outside a contract");
    let value = mlir_op!(context.state, &block, ThisOperation.addr(contract_type));
    BlockAnd { block, value }
});

expression_emit!(StringExpression; |node, context, block| {
    let bytes = node.value();
    let text = unsafe { std::str::from_utf8_unchecked(&bytes) };
    let value = AstValue::string_literal(text, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

/// Emits a string literal coerced to a `byte` / `bytesN` target as a compile-time constant.
///
/// A `byte` / `bytesN` value is left-aligned: the literal fills the high bytes, zero-padded on the
/// right. The natural [`StringExpression::emit`] produces a runtime `!sol.string`; toward a
/// fixed-width byte target that value is materialised directly as a `sol.constant` then narrowed
/// with `sol.bytes_cast`. Any other target falls back to the runtime string emission.
impl<'context: 'block, 'block> EmitAs<'context, 'block, Type<'context>> for StringExpression {
    type Output = Value<'context, 'block>;

    fn emit_as<'state>(
        &self,
        target: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let Some(width) = context.fixed_bytes_or_byte_width(target) else {
            return self.emit(context, block);
        };
        let bytes = self.value();
        let mut buffer = vec![0u8; width as usize];
        for (slot, byte) in buffer.iter_mut().zip(bytes.iter()) {
            *slot = *byte;
        }
        let integer_type = AstType::unsigned(
            context.state.mlir_context,
            width as usize * solx_utils::BIT_LENGTH_BYTE,
        );
        let integer = AstValue::constant_from_bigint(
            &BigInt::from_bytes_be(Sign::Plus, &buffer),
            integer_type,
            context.state,
            &block,
        );
        let value = integer
            .bytes_cast(AstType::new(target), context.state, &block)
            .into_mlir();
        BlockAnd { block, value }
    }
}
