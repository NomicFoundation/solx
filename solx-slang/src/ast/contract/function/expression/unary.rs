//!
//! Unary expression lowering: prefix and postfix operators. Each node bridges
//! to the [`Operator`] it applies, which lowers itself.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::PrefixExpressionOperator;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::ExpressionExt;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::type_conversion::TypeConversion;

expression_emit!(PostfixExpression; |node, context, block| {
    // Peel parenthesised single-element tuples so `(i)++` / `(arr[j])--` resolve
    // their lvalue exactly like the bare `i++` / `arr[j]--`.
    let operand = node.operand().unwrap_parentheses();
    let operator = match node.operator() {
        PostfixExpressionOperator::MinusMinus(_) => Operator::Decrement,
        PostfixExpressionOperator::PlusPlus(_) => Operator::Increment,
    };
    let (value, block) = operator.emit_postfix(context, &operand, block)?;
    Ok(BlockAnd { block, value })
});

expression_emit!(PrefixExpression; |node, context, block| {
    // `delete x` is value-less, so it never reaches value position: a statement
    // discard site emits it directly (`emit_delete`), never through this `Emit`.
    if let PrefixExpressionOperator::DeleteKeyword(_) = node.operator() {
        unreachable!("`delete` is value-less; a discard site emits it, not value-position `Emit`");
    }
    let result_type =
        TypeConversion::resolve_optional_slang_type(node.get_type(), &context.state.builder);
    let operator = match node.operator() {
        PrefixExpressionOperator::Bang(_) => Operator::Not,
        PrefixExpressionOperator::DeleteKeyword(_) => {
            unreachable!("delete is routed before prefix-operator classification")
        }
        PrefixExpressionOperator::Minus(_) => Operator::Subtract,
        PrefixExpressionOperator::MinusMinus(_) => Operator::Decrement,
        PrefixExpressionOperator::PlusPlus(_) => Operator::Increment,
        PrefixExpressionOperator::Tilde(_) => Operator::BitwiseNot,
    };
    // Peel parenthesised single-element tuples so `--(i)` / `~(x)` operate on the
    // bare inner lvalue / value, as solc treats them.
    let operand = node.operand().unwrap_parentheses();
    let (value, block) = operator.emit_prefix(context, &operand, result_type, block)?;
    Ok(BlockAnd { block, value })
});
