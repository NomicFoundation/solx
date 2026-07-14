//!
//! The short-circuit logical operators, lowered through the ternary's value-branch: the result is
//! initialized with the value the left operand alone decides, and the right operand is evaluated
//! in the single arm the left operand does not short-circuit.
//!

use solx_mlir::Type;
use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::contract::function::expression::conditional::ConditionalExpression;

codegen!(
    /// `a && b`: `false` unless `a` holds, so the result is initialized `false` and `b` is
    /// evaluated in the then-arm, the else-arm keeping the initializer.
    AndExpression -> Value |node, scope| {
        let condition = Expression::emit(&node.left_operand(), scope).is_nonzero(scope);
        let result_type = Type::boolean(scope.melior);
        ConditionalExpression::branch_value(
            scope,
            condition,
            result_type,
            |scope| Some(Value::boolean(false, scope)),
            |scope| Some(Expression::emit(&node.right_operand(), scope).is_nonzero(scope)),
            |_context| None,
        )
    }

    /// `a || b`: `true` when `a` holds, so the result is initialized `true` and `b` is evaluated in
    /// the else-arm, the then-arm keeping the initializer.
    OrExpression -> Value |node, scope| {
        let condition = Expression::emit(&node.left_operand(), scope).is_nonzero(scope);
        let result_type = Type::boolean(scope.melior);
        ConditionalExpression::branch_value(
            scope,
            condition,
            result_type,
            |scope| Some(Value::boolean(true, scope)),
            |_context| None,
            |scope| Some(Expression::emit(&node.right_operand(), scope).is_nonzero(scope)),
        )
    }
);
