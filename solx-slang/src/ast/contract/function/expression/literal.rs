//!
//! Literal and primary-keyword expression emission: number / boolean / string
//! literals and the `this` keyword.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::FalseKeyword;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::ThisKeyword;
use slang_solidity_v2::ast::TrueKeyword;
use solx_mlir::ods::sol::ThisOperation;
use solx_utils::BIT_LENGTH_BYTE;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(DecimalNumberExpression, HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("slang validated");
    let result_type =
        AstType::resolve_optional(node.get_type(), context.state)
            .expect("slang validated");
    let constant = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        context.state,
        &block,
    );
    BlockAnd {
        block,
        value: constant,
    }
});

expression_emit!(TrueKeyword; |context, block| {
    let value = AstValue::boolean(true, context.state, &block);
    BlockAnd { block, value }
});

expression_emit!(FalseKeyword; |context, block| {
    let value = AstValue::boolean(false, context.state, &block);
    BlockAnd { block, value }
});

expression_emit!(ThisKeyword; |context, block| {
    let contract_type = context
        .state
        .current_contract_type
        .expect("slang validated");
    let value: Value<'context, 'block> =
        mlir_op!(context.state, block, ThisOperation.addr(contract_type));
    BlockAnd {
        block,
        value: value.into(),
    }
});

expression_emit!(StringExpression; |node, context, block| {
    let bytes = node.value();
    // The bytes are read only as bytes by `StringAttribute::new`, never as UTF-8 (a Solidity
    // literal may be non-UTF-8, e.g. `"\xff"`), so the unchecked conversion is sound and a checked
    // `from_utf8` would wrongly reject valid input.
    let literal = unsafe { std::str::from_utf8_unchecked(&bytes) };
    let value = AstValue::string_literal(literal, context.state, &block);
    BlockAnd { block, value }
});

impl<'context: 'block, 'block> EmitAs<'context, 'block, Type<'context>> for StringExpression {
    type Output = AstValue<'context, 'block>;

    fn emit_as<'state>(
        &self,
        target_type: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, AstValue<'context, 'block>> {
        let state = context.state;
        if AstType::new(target_type).is_byte() {
            let byte = self.value().first().copied().unwrap_or(0);
            let ui8 = Type::from(IntegerType::unsigned(state.mlir(), BIT_LENGTH_BYTE as u32));
            let integer = AstValue::constant_from_bigint(
                &BigInt::from(byte),
                AstType::new(ui8),
                state,
                &block,
            );
            let value = integer.cast(AstType::new(target_type), state, &block);
            return BlockAnd { block, value };
        }
        if let Some(width) = AstType::new(target_type).fixed_bytes_or_byte_width() {
            let mut buffer = vec![0u8; width as usize];
            for (slot, byte) in buffer.iter_mut().zip(self.value().iter()) {
                *slot = *byte;
            }
            let integer_value = BigInt::from_bytes_be(Sign::Plus, &buffer);
            let integer_type = Type::from(IntegerType::unsigned(
                state.mlir(),
                width * BIT_LENGTH_BYTE as u32,
            ));
            let integer = AstValue::constant_from_bigint(
                &integer_value,
                AstType::new(integer_type),
                state,
                &block,
            );
            let value = integer.cast(AstType::fixed_bytes(state.mlir(), width), state, &block);
            return BlockAnd { block, value };
        }
        self.emit(context, block)
    }
}
