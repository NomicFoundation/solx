//!
//! Literal and primary-keyword expression emission: number / boolean / string
//! literals and the `this` keyword.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::FalseKeyword;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::ThisKeyword;
use slang_solidity_v2::ast::TrueKeyword;
use solx_mlir::ods::sol::ThisOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

// A decimal and a hex integer literal lower identically: slang has already computed the integer
// value, so both materialise a typed constant at the binder's literal type.
expression_emit!(DecimalNumberExpression, HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("slang validated");
    let result_type =
        AstType::resolve_optional(node.get_type(), &context.state.builder)
            .expect("slang validated");
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
        .expect("slang validated");
    let value: Value<'context, 'block> =
        mlir_op!(&context.state.builder, block, ThisOperation.addr(contract_type));
    BlockAnd {
        block,
        value: value.into(),
    }
});

expression_emit!(StringExpression; |node, context, block| {
    let bytes = node.value();
    let literal = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
    let value = AstValue::string_literal(literal, &context.state.builder, &block);
    BlockAnd { block, value }
});
