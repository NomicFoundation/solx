//!
//! The prefix and postfix unary operators.
//!

use slang_solidity_v2::ast::Expression as SlangExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpressionOperator;

use solx_mlir::CmpPredicate;
use solx_mlir::Value;

use crate::contract::function::expression::Expression;

codegen!(
    /// The prefix `++`, `--`, `~`, `!`, and `-` operators.
    PrefixExpression -> Value |node, scope| {
        match node.operator() {
            PrefixExpressionOperator::PlusPlus(_) => {
                Expression::step(&node.operand(), Value::add, scope).1
            }
            PrefixExpressionOperator::MinusMinus(_) => {
                Expression::step(&node.operand(), Value::subtract, scope).1
            }
            PrefixExpressionOperator::Tilde(_) => {
                let result_type = codegen!(@result_type PrefixExpression, node, scope);
                Expression::emit(&node.operand(), scope)
                    .coerce(result_type, scope)
                    .not(scope)
            }
            PrefixExpressionOperator::Bang(_) => {
                let value = Expression::emit(&node.operand(), scope);
                let zero = Value::zero(value.r#type(), scope);
                value.compare(zero, CmpPredicate::Eq, scope)
            }
            PrefixExpressionOperator::Minus(_) => {
                let result_type = codegen!(@result_type PrefixExpression, node, scope);
                let magnitude = match node.operand() {
                    SlangExpression::DecimalNumberExpression(number) => number.integer_value(),
                    SlangExpression::HexNumberExpression(number) => number.integer_value(),
                    _ => None,
                };
                match magnitude {
                    Some(magnitude) => {
                        Value::constant_from_bigint(&-magnitude, result_type, scope)
                    }
                    None => {
                        let value = Expression::emit(&node.operand(), scope)
                            .coerce(result_type, scope);
                        Value::zero(result_type, scope).subtract(value, scope.checked(), scope)
                    }
                }
            }
            PrefixExpressionOperator::DeleteKeyword(_) => {
                unimplemented!("delete expression is not yet supported")
            }
        }
    }

    /// The postfix `++` and `--` operators, yielding the value before the step.
    PostfixExpression -> Value |node, scope| {
        Expression::step(
            &node.operand(),
            match node.operator() {
                PostfixExpressionOperator::PlusPlus(_) => Value::add,
                PostfixExpressionOperator::MinusMinus(_) => Value::subtract,
            },
            scope,
        )
        .0
    }
);
