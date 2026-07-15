//!
//! The short-circuit logical operators, lowered through the ternary's value-branch: the result is
//! initialized with the value the left operand alone decides, and the right operand is evaluated in
//! the single arm the left operand does not short-circuit.
//!

use slang_solidity_v2::ast::AndExpression;
use slang_solidity_v2::ast::OrExpression;

use solx_mlir::Type;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `a && b`: `false` unless `a` holds, so the result is initialized `false` and `b` is evaluated
    /// in the then-arm, the else-arm keeping the initializer.
    pub fn and(&mut self, node: &AndExpression) -> Value<'context> {
        let condition = self.expression(&node.left_operand()).is_nonzero(self);
        self.branch_value(
            condition,
            Type::boolean(self.melior),
            |scope| Some(Value::boolean(false, scope)),
            |scope| Some(scope.expression(&node.right_operand()).is_nonzero(scope)),
            |_scope| None,
        )
    }

    /// `a || b`: `a` itself is the result when it holds, so the result is initialized with the left
    /// operand's truthiness and `b` is evaluated in the else-arm, the then-arm keeping the
    /// initializer.
    pub fn or(&mut self, node: &OrExpression) -> Value<'context> {
        let condition = self.expression(&node.left_operand()).is_nonzero(self);
        self.branch_value(
            condition,
            Type::boolean(self.melior),
            |_scope| Some(condition),
            |_scope| None,
            |scope| Some(scope.expression(&node.right_operand()).is_nonzero(scope)),
        )
    }
}
