//!
//! Literal and primary-keyword expression emission: number / boolean / string
//! literals and the `this` keyword.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
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
    let text = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
    let value = AstValue::string_literal(text, context.state, &block).into_mlir();
    BlockAnd { block, value }
});
