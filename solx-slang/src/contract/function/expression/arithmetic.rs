//!
//! Arithmetic expressions: the additive, multiplicative, and exponentiation operators.
//!

use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::AdditiveExpressionOperator;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::MultiplicativeExpressionOperator;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `a + b` and `a - b`, both operands coerced to the binder's result type.
    pub fn additive(&mut self, node: &AdditiveExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        match node.operator() {
            AdditiveExpressionOperator::Plus(_) => lhs.add(rhs, self.checked, self),
            AdditiveExpressionOperator::Minus(_) => lhs.subtract(rhs, self.checked, self),
        }
    }

    /// `a * b`, `a / b`, and `a % b`, both operands coerced to the binder's result type.
    pub fn multiplicative(&mut self, node: &MultiplicativeExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        match node.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => lhs.multiply(rhs, self.checked, self),
            MultiplicativeExpressionOperator::Slash(_) => lhs.divide(rhs, self.checked, self),
            MultiplicativeExpressionOperator::Percent(_) => lhs.remainder(rhs, self),
        }
    }

    /// `a ** b`, both operands coerced to the binder's result type.
    pub fn exponentiation(&mut self, node: &ExponentiationExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.coerced_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        lhs.exponentiate(rhs, self.checked, self)
    }
}
