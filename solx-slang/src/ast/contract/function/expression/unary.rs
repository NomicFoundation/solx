//!
//! Unary expression emission: prefix and postfix operators. Each node bridges
//! to the [`Operator`] it applies, which lowers itself.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::PrefixExpressionOperator;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;

expression_emit!(PostfixExpression; |node, context, block| {
    let operand = node.operand().unwrap_parentheses();
    let operator = match node.operator() {
        PostfixExpressionOperator::MinusMinus(_) => Operator::Decrement,
        PostfixExpressionOperator::PlusPlus(_) => Operator::Increment,
    };
    let (value, block) = operator.emit_postfix(context, &operand, block);
    BlockAnd { block, value }
});

expression_emit!(PrefixExpression; |node, context, block| {
    if let PrefixExpressionOperator::DeleteKeyword(_) = node.operator() {
        unreachable!("`delete` is value-less; a discard site emits it, not value-position `Emit`");
    }
    let result_type =
        AstType::resolve_optional(node.get_type(), context.state);
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
    let operand = node.operand().unwrap_parentheses();
    let (value, block) = operator.emit_prefix(context, &operand, result_type, block);
    BlockAnd { block, value }
});
