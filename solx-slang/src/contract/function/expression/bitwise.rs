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
    /// `a & b`, both operands converted to the binder's result type.
    pub fn bitwise_and(&mut self, node: &BitwiseAndExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.converted_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitand(rhs, self)
    }

    /// `a | b`, both operands converted to the binder's result type.
    pub fn bitwise_or(&mut self, node: &BitwiseOrExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.converted_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitor(rhs, self)
    }

    /// `a ^ b`, both operands converted to the binder's result type.
    pub fn bitwise_xor(&mut self, node: &BitwiseXorExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.converted_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.bitxor(rhs, self)
    }

    /// `a << b` and `a >> b`, the shifted value converted to the binder's result type and the shift
    /// amount kept at its own type.
    pub fn shift(&mut self, node: &ShiftExpression) -> Value<'context> {
        let result_type = self.typing(node.get_type());
        let amount = self.expression(&node.right_operand());
        let value = self.converted(&node.left_operand(), result_type);
        match node.operator() {
            ShiftExpressionOperator::LessThanLessThan(_) => value.shl(amount, self),
            ShiftExpressionOperator::GreaterThanGreaterThan(_) => value.shr(amount, self),
            ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => {
                unreachable!("Solidity has no >>> operator")
            }
        }
    }
}
