//!
//! Bitwise expressions: the and, or, xor, and shift operators.
//!

use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::ShiftExpression;
use slang_solidity_v2::ast::ShiftExpressionOperator;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `a & b`, both operands coerced to the binder's result type.
    pub fn bitwise_and(&mut self, node: &BitwiseAndExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitand(rhs, self)
    }

    /// `a | b`, both operands coerced to the binder's result type.
    pub fn bitwise_or(&mut self, node: &BitwiseOrExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitor(rhs, self)
    }

    /// `a ^ b`, both operands coerced to the binder's result type.
    pub fn bitwise_xor(&mut self, node: &BitwiseXorExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitxor(rhs, self)
    }

    /// `a << b` and `a >> b`, both operands coerced to the binder's result type.
    pub fn shift(&mut self, node: &ShiftExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        match node.operator() {
            ShiftExpressionOperator::LessThanLessThan(_) => lhs.shl(rhs, self),
            ShiftExpressionOperator::GreaterThanGreaterThan(_)
            | ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => lhs.shr(rhs, self),
        }
    }
}
