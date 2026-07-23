//!
//! Arithmetic expressions: the additive, multiplicative, and exponentiation operators, and the
//! `addmod`/`mulmod` modular built-ins.
//!

use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::AdditiveExpressionOperator;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::MultiplicativeExpressionOperator;
use slang_solidity_v2::ast::PositionalArguments;

use solx_mlir::Context;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `a + b` and `a - b`, both operands converted to the binder's result type.
    pub fn additive(&mut self, node: &AdditiveExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.converted_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        match node.operator() {
            AdditiveExpressionOperator::Plus(_) => lhs.add(rhs, self.checked, self),
            AdditiveExpressionOperator::Minus(_) => lhs.subtract(rhs, self.checked, self),
        }
    }

    /// `a * b`, `a / b`, and `a % b`, both operands converted to the binder's result type.
    pub fn multiplicative(&mut self, node: &MultiplicativeExpression) -> Value<'context> {
        let (lhs, rhs) =
            self.converted_operands(node.get_type(), &node.left_operand(), &node.right_operand());
        match node.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => lhs.multiply(rhs, self.checked, self),
            MultiplicativeExpressionOperator::Slash(_) => lhs.divide(rhs, self.checked, self),
            MultiplicativeExpressionOperator::Percent(_) => lhs.remainder(rhs, self),
        }
    }

    /// `a ** b`, the base converted to the binder's result type and the exponent kept at its own type.
    pub fn exponentiation(&mut self, node: &ExponentiationExpression) -> Value<'context> {
        let result_type = self.typing(node.get_type());
        let exponent = self.expression(&node.right_operand());
        let base = self.converted(&node.left_operand(), result_type);
        base.exponentiate(exponent, self.checked, self)
    }

    /// The shared `addmod`/`mulmod` lowering: the three operands widened to the field width the
    /// built-in works at rather than their own narrower types, evaluated right-to-left to match
    /// legacy, then combined by `operator`.
    pub fn modular(
        &mut self,
        arguments: &PositionalArguments,
        operator: impl FnOnce(
            Value<'context>,
            Value<'context>,
            Value<'context>,
            &Context<'context>,
        ) -> Value<'context>,
    ) -> Value<'context> {
        let field = MlirType::field(self.melior);
        let arguments: Vec<_> = arguments.iter().collect();
        let modulus = self.converted(&arguments[2], field);
        let right = self.converted(&arguments[1], field);
        let left = self.converted(&arguments[0], field);
        operator(left, right, modulus, self)
    }
}
