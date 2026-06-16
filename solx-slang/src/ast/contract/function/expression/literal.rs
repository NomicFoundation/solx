//!
//! Literal and primary-keyword expression emission: number / boolean / string
//! literals and the `this` keyword.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::FalseKeyword;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::ThisKeyword;
use slang_solidity_v2::ast::TrueKeyword;
use solx_mlir::ods::sol::StringLitOperation;
use solx_mlir::ods::sol::ThisOperation;
use solx_utils::BIT_LENGTH_BYTE;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Materialize;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

// A decimal and a hex integer literal lower identically: slang has already
// computed the integer value (decimals after unit/denomination scaling, hex
// verbatim), so both materialise a typed constant at the binder's literal type.
expression_emit!(DecimalNumberExpression, HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("an integer literal evaluates to an integer after units");
    let result_type =
        AstType::resolve_optional(node.get_type(), &context.state.builder)
            .expect("the binder types every integer literal node");
    let constant = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        &context.state.builder,
        &block,
    );
    BlockAnd {
        block,
        value: constant,
    }
});

expression_emit!(TrueKeyword; |context, block| {
    let value = AstValue::boolean(true, &context.state.builder, &block);
    BlockAnd { block, value }
});

expression_emit!(FalseKeyword; |context, block| {
    let value = AstValue::boolean(false, &context.state.builder, &block);
    BlockAnd { block, value }
});

expression_emit!(ThisKeyword; |context, block| {
    let contract_type = context
        .state
        .current_contract_type
        .expect("`this` only appears inside a contract method");
    let value: Value<'context, 'block> =
        sol_op!(&context.state.builder, block, ThisOperation.addr(contract_type));
    BlockAnd {
        block,
        value: value.into(),
    }
});

expression_emit!(StringExpression; |node, context, block| {
    // A string literal's bytes are emitted verbatim — they need not be valid
    // UTF-8 (`hex"..."`, `"\xff"`).
    let bytes = node.value();
    let builder = &context.state.builder;
    // the `&str` is only consumed by `StringAttribute::new`, which hands it
    // to `StringRef::new` — that reads `.as_ptr()`/`.len()` and never assumes UTF-8
    // validity, so the non-UTF-8 literal bytes are sound here.
    let literal = unsafe { std::str::from_utf8_unchecked(&bytes) };
    let value: Value<'context, 'block> = sol_op!(
        builder,
        &block,
        StringLitOperation
            .value(StringAttribute::new(builder.context, literal))
            .addr(AstType::string(builder.context, solx_utils::DataLocation::Memory))
    );
    BlockAnd {
        block,
        value: value.into(),
    }
});

// A string literal used where `bytesN` / `byte` is expected materialises toward
// that type as a compile-time fixed-bytes / byte constant rather than the runtime
// `sol.string` its natural `Emit` produces. The impl lives here, beside that
// `Emit`, because both read `ExpressionContext`'s private state.
impl<'state, 'context, 'block, 'scope> Materialize<'context, 'block, 'state, 'scope>
    for StringExpression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;

    fn materialize(
        &self,
        target_type: Type<'context>,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, AstValue<'context, 'block>> {
        let builder = &context.state.builder;
        // A string literal toward a single `byte` (an element of `bytes` /
        // `string`) materialises as a `!sol.byte` constant.
        if AstType::new(target_type).is_byte() {
            let byte = self.value().first().copied().unwrap_or(0); // recut-lint-allow: fail01 — an empty string literal toward a byte is 0x00 (zero-padding)
            let ui8 = Type::from(IntegerType::unsigned(
                builder.context,
                BIT_LENGTH_BYTE as u32,
            ));
            let integer = AstValue::constant_from_bigint(
                &BigInt::from(byte),
                AstType::new(ui8),
                builder,
                &block,
            );
            let value = integer.cast(AstType::new(target_type), builder, &block);
            return BlockAnd { block, value };
        }
        // `bytesN` is left-aligned: the literal occupies the high bytes,
        // zero-padded on the right.
        if let Some(width) = AstType::new(target_type).fixed_bytes_or_byte_width() {
            let mut buffer = vec![0u8; width as usize];
            for (slot, byte) in buffer.iter_mut().zip(self.value().iter()) {
                *slot = *byte;
            }
            let integer_value = BigInt::from_bytes_be(Sign::Plus, &buffer);
            let integer_type = Type::from(IntegerType::unsigned(
                builder.context,
                width * BIT_LENGTH_BYTE as u32,
            ));
            let integer = AstValue::constant_from_bigint(
                &integer_value,
                AstType::new(integer_type),
                builder,
                &block,
            );
            let value = integer.cast(
                AstType::fixed_bytes(builder.context, width),
                builder,
                &block,
            );
            return BlockAnd { block, value };
        }
        self.emit(context, block)
    }
}
