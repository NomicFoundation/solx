//!
//! Tuple expression emission: a single-element tuple forwards to its inner expression.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::TupleExpression;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(TupleExpression; |node, context, block| {
    let items = node.items();
    let item = items.iter().next().expect("slang validates non-empty tuple");
    let inner = item.expression().expect("tuple element is non-empty");
    inner.emit(context, block)
});
