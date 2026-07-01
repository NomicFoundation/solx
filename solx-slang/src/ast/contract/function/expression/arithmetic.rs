//!
//! Arithmetic expression emission: additive, multiplicative, exponentiation, bitwise, and shift operations.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::AdditiveExpressionOperator;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::MultiplicativeExpressionOperator;
use slang_solidity_v2::ast::ShiftExpression;
use slang_solidity_v2::ast::ShiftExpressionOperator;

use solx_mlir::Type as AstType;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_expression::EmitExpression;

impl From<&AdditiveExpression> for Operator {
    fn from(node: &AdditiveExpression) -> Self {
        match node.operator() {
            AdditiveExpressionOperator::Plus(_) => Self::Add,
            AdditiveExpressionOperator::Minus(_) => Self::Subtract,
        }
    }
}

impl From<&MultiplicativeExpression> for Operator {
    fn from(node: &MultiplicativeExpression) -> Self {
        match node.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => Self::Multiply,
            MultiplicativeExpressionOperator::Percent(_) => Self::Remainder,
            MultiplicativeExpressionOperator::Slash(_) => Self::Divide,
        }
    }
}

impl From<&ExponentiationExpression> for Operator {
    fn from(_node: &ExponentiationExpression) -> Self {
        Self::Exponentiation
    }
}

impl From<&BitwiseAndExpression> for Operator {
    fn from(_node: &BitwiseAndExpression) -> Self {
        Self::BitwiseAnd
    }
}

impl From<&BitwiseOrExpression> for Operator {
    fn from(_node: &BitwiseOrExpression) -> Self {
        Self::BitwiseOr
    }
}

impl From<&BitwiseXorExpression> for Operator {
    fn from(_node: &BitwiseXorExpression) -> Self {
        Self::BitwiseXor
    }
}

impl From<&ShiftExpression> for Operator {
    fn from(node: &ShiftExpression) -> Self {
        match node.operator() {
            ShiftExpressionOperator::GreaterThanGreaterThan(_) => Self::ShiftRight,
            ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => {
                unreachable!(">>> is not a valid Solidity operator")
            }
            ShiftExpressionOperator::LessThanLessThan(_) => Self::ShiftLeft,
        }
    }
}

expression_emit!(
    AdditiveExpression,
    MultiplicativeExpression,
    ExponentiationExpression,
    BitwiseAndExpression,
    BitwiseOrExpression,
    BitwiseXorExpression,
    ShiftExpression;
    |node, context, block| {
        let result_type =
            AstType::resolve_optional(node.get_type(), context.state);
        let (value, block) = Operator::from(node).emit_binary(
            context,
            &node.left_operand(),
            &node.right_operand(),
            result_type,
            block,
        );
        BlockAnd { block, value }
    }
);
