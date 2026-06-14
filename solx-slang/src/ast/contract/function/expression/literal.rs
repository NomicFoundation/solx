//!
//! Literal and primary-keyword expression lowering: number / boolean / string
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
use slang_solidity_v2::ast::Expression;
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
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::type_conversion::TypeConversion;

// A decimal and a hex integer literal lower identically: slang has already
// computed the integer value (decimals after unit/denomination scaling, hex
// verbatim), so both materialise a typed constant at the binder's literal type.
expression_emit!(DecimalNumberExpression, HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("an integer literal evaluates to an integer after units");
    let result_type =
        TypeConversion::resolve_optional_slang_type(node.get_type(), &context.state.builder)
            .expect("the binder types every integer literal node");
    let constant = context
        .state
        .builder
        .emit_constant(&value, result_type, &block);
    Ok(BlockAnd {
        block,
        value: constant.into(),
    })
});

expression_emit!(TrueKeyword; |context, block| {
    let value = context.state.builder.emit_bool(true, &block);
    Ok(BlockAnd {
        block,
        value: value.into(),
    })
});

expression_emit!(FalseKeyword; |context, block| {
    let value = context.state.builder.emit_bool(false, &block);
    Ok(BlockAnd {
        block,
        value: value.into(),
    })
});

expression_emit!(ThisKeyword; |context, block| {
    let contract_type = context
        .state
        .current_contract_type
        .expect("`this` only appears inside a contract method");
    let value: Value<'context, 'block> =
        sol_op!(&context.state.builder, block, ThisOperation.addr(contract_type));
    Ok(BlockAnd {
        block,
        value: value.into(),
    })
});

expression_emit!(StringExpression; |node, context, block| {
    // A string literal's bytes are emitted verbatim — they need not be valid
    // UTF-8 (`hex"..."`, `"\xff"`).
    let bytes = node.value();
    let builder = &context.state.builder;
    // SAFETY: the `&str` is only consumed by `StringAttribute::new`, which hands it
    // to `StringRef::new` — that reads `.as_ptr()`/`.len()` and never assumes UTF-8
    // validity, so the non-UTF-8 literal bytes are sound here.
    let literal = unsafe { std::str::from_utf8_unchecked(&bytes) };
    let value: Value<'context, 'block> = sol_op!(
        builder,
        &block,
        StringLitOperation
            .value(StringAttribute::new(builder.context, literal))
            .addr(crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir())
    );
    Ok(BlockAnd {
        block,
        value: value.into(),
    })
});

/// An expression emitted toward an expected MLIR type.
///
/// The one case where the natural emission is wrong: a string literal used where
/// `bytesN` / `byte` is expected (`bytes7 x = "abc"`, `b == "1234567"`, an element
/// `s[i] = "c"`) is a compile-time fixed-bytes constant, not a runtime
/// `sol.string` — emitting the string and casting it fails the integer-only
/// verifier. slang types the literal `Literal(String)` regardless of context
/// (slang#1793), so the target reaches the literal only from the use site, here.
/// `bytesN` is left-aligned: the literal occupies the high bytes, zero-padded on
/// the right. Every other expression emits naturally (the caller coerces), so a
/// coercion site routed through this is a pure superset of [`Emit::emit`].
pub struct Toward<'expression, 'context> {
    /// The expression to emit.
    pub expression: &'expression Expression,
    /// The MLIR type it is emitted toward.
    pub target_type: Type<'context>,
}

impl<'expression, 'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope>
    for Toward<'expression, 'context>
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = BlockAnd<'context, 'block, crate::ast::Value<'context, 'block>>;

    fn emit(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Self::Output> {
        if let Expression::StringExpression(string_expression) = self.expression {
            let builder = &context.state.builder;
            // A string literal toward a single `byte` (an element of
            // `bytes`/`string`) materialises as a `!sol.byte` constant.
            if crate::ast::Type::new(self.target_type).is_byte() {
                let byte = string_expression.value().first().copied().unwrap_or(0);
                let ui8 = Type::from(IntegerType::unsigned(
                    builder.context,
                    BIT_LENGTH_BYTE as u32,
                ));
                let integer = builder.emit_constant(&BigInt::from(byte), ui8, &block);
                let value =
                    crate::ast::Value::from(integer).cast(self.target_type, builder, &block);
                return Ok(BlockAnd { block, value });
            }
            if let Some(width) = crate::ast::Type::new(self.target_type).fixed_bytes_or_byte_width()
            {
                let mut buffer = vec![0u8; width as usize];
                for (slot, byte) in buffer.iter_mut().zip(string_expression.value().iter()) {
                    *slot = *byte;
                }
                let integer_value = BigInt::from_bytes_be(Sign::Plus, &buffer);
                let integer_type = Type::from(IntegerType::unsigned(
                    builder.context,
                    width * BIT_LENGTH_BYTE as u32,
                ));
                let integer = builder.emit_constant(&integer_value, integer_type, &block);
                let value = crate::ast::Value::from(integer).cast(
                    crate::ast::Type::fixed_bytes(builder.context, width).into_mlir(),
                    builder,
                    &block,
                );
                return Ok(BlockAnd { block, value });
            }
        }
        self.expression.emit(context, block)
    }
}
