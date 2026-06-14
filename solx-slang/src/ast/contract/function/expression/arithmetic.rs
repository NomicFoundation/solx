//!
//! Arithmetic expression lowering: binary additive, multiplicative,
//! exponentiation, bitwise, and shift operations. Each node bridges to the
//! [`Operator`] it applies, which lowers itself.
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

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::type_conversion::TypeConversion;

/// Bridges a slang binary-expression node to the [`Operator`] it applies, so the
/// shared binary lowering body handles all of them uniformly. Each node maps its
/// typed slang operator enum (or its single fixed operator) — never source text.
trait BinaryOperatorExt {
    /// The [`Operator`] this binary expression applies.
    fn bridged_operator(&self) -> Operator;
}

impl BinaryOperatorExt for AdditiveExpression {
    fn bridged_operator(&self) -> Operator {
        match self.operator() {
            AdditiveExpressionOperator::Plus(_) => Operator::Add,
            AdditiveExpressionOperator::Minus(_) => Operator::Subtract,
        }
    }
}

impl BinaryOperatorExt for MultiplicativeExpression {
    fn bridged_operator(&self) -> Operator {
        match self.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => Operator::Multiply,
            MultiplicativeExpressionOperator::Percent(_) => Operator::Remainder,
            MultiplicativeExpressionOperator::Slash(_) => Operator::Divide,
        }
    }
}

impl BinaryOperatorExt for ExponentiationExpression {
    fn bridged_operator(&self) -> Operator {
        Operator::Exponentiation
    }
}

impl BinaryOperatorExt for BitwiseAndExpression {
    fn bridged_operator(&self) -> Operator {
        Operator::BitwiseAnd
    }
}

impl BinaryOperatorExt for BitwiseOrExpression {
    fn bridged_operator(&self) -> Operator {
        Operator::BitwiseOr
    }
}

impl BinaryOperatorExt for BitwiseXorExpression {
    fn bridged_operator(&self) -> Operator {
        Operator::BitwiseXor
    }
}

impl BinaryOperatorExt for ShiftExpression {
    fn bridged_operator(&self) -> Operator {
        match self.operator() {
            ShiftExpressionOperator::GreaterThanGreaterThan(_) => Operator::ShiftRight,
            ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => Operator::ShiftRight,
            ShiftExpressionOperator::LessThanLessThan(_) => Operator::ShiftLeft,
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
        // The result type slang assigns the expression annotates the operands and
        // result (matching solc); a `None` lets `emit_binary` pick the wider one.
        let result_type =
            TypeConversion::resolve_optional_slang_type(node.get_type(), &context.state.builder);
        let (value, block) = node.bridged_operator().emit_binary(
            context,
            &node.left_operand(),
            &node.right_operand(),
            result_type,
            block,
        )?;
        Ok(BlockAnd {
            block,
            value: value.into(),
        })
    }
);
