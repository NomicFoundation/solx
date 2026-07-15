//!
//! The prefix and postfix unary operators.
//!

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::PrefixExpressionOperator;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The prefix `++`, `--`, `~`, `!`, and `-` operators.
    pub fn prefix(&mut self, node: &PrefixExpression) -> Value<'context> {
        match node.operator() {
            PrefixExpressionOperator::PlusPlus(_) => self.step(&node.operand(), Value::add).1,
            PrefixExpressionOperator::MinusMinus(_) => {
                self.step(&node.operand(), Value::subtract).1
            }
            PrefixExpressionOperator::Tilde(_) => {
                let result_type = self.typing(node.get_type());
                self.expression(&node.operand())
                    .coerce(result_type, self)
                    .not(self)
            }
            PrefixExpressionOperator::Bang(_) => {
                let value = self.expression(&node.operand());
                value.compare(Value::zero(value.r#type(), self), CmpPredicate::Eq, self)
            }
            PrefixExpressionOperator::Minus(_) => {
                let result_type = self.typing(node.get_type());
                let magnitude = match node.operand() {
                    Expression::DecimalNumberExpression(number) => number.integer_value(),
                    Expression::HexNumberExpression(number) => number.integer_value(),
                    _ => None,
                };
                match magnitude {
                    Some(magnitude) => Value::constant_from_bigint(&-magnitude, result_type, self),
                    None => {
                        let value = self.expression(&node.operand()).coerce(result_type, self);
                        Value::zero(result_type, self).subtract(value, self.checked, self)
                    }
                }
            }
            PrefixExpressionOperator::DeleteKeyword(_) => {
                unimplemented!("delete expression is not yet supported")
            }
        }
    }

    /// The postfix `++` and `--` operators, yielding the value before the step.
    pub fn postfix(&mut self, node: &PostfixExpression) -> Value<'context> {
        self.step(
            &node.operand(),
            match node.operator() {
                PostfixExpressionOperator::PlusPlus(_) => Value::add,
                PostfixExpressionOperator::MinusMinus(_) => Value::subtract,
            },
        )
        .0
    }

    /// The shared `++`/`--` lowering: load the operand's place, apply the stepping operator to its
    /// value and one, store back, returning the value before and after the step.
    fn step(
        &mut self,
        operand: &Expression,
        operator: impl FnOnce(
            Value<'context>,
            Value<'context>,
            bool,
            &Context<'context>,
        ) -> Value<'context>,
    ) -> (Value<'context>, Value<'context>) {
        let (place, element_type) = self.expression_place(operand);
        let old = place.load(element_type, self);
        let new = operator(old, Value::one(element_type, self), self.checked, self);
        place.store(new, self);
        (old, new)
    }
}
