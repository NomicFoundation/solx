//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context> ExpressionEmitter<'state, 'context> {
    /// Emits a binary arithmetic Sol dialect operation.
    ///
    /// When `target_type` is `Some`, both operands are cast to that type and
    /// the result has that type. When `None`, selects the wider operand type
    /// by bit width.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: Operator,
        target_type: Option<Type<'context>>,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let rhs = self.emit_value(right, context)?;
        let lhs = self.emit_value(left, context)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = lhs.r#type().integer_bit_width();
            let rhs_width = rhs.r#type().integer_bit_width();
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs = TypeConversion::from_target_type(result_type, context).emit(lhs, context);
        let rhs = TypeConversion::from_target_type(result_type, context).emit(rhs, context);
        let value = operator.emit(self.checked, lhs, rhs, context);
        Ok(value)
    }

    /// Emits postfix `++` or `--`, returning the old value.
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        context: &Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let (old, _) = self.emit_increment_decrement(operand, operator, context)?;
        Ok(old)
    }

    /// Emits prefix operators: `!`, `-`, `~`, `++`, `--`.
    ///
    /// When `target_type` is `Some`, unary operations use that type. When
    /// `None`, falls back to ui256 semantics.
    pub fn emit_prefix(
        &self,
        operator: Operator,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        match operator {
            Operator::Increment | Operator::Decrement => {
                let (_old, new_value) =
                    self.emit_increment_decrement(operand, operator, context)?;
                Ok(new_value)
            }
            Operator::BitwiseNot => {
                let value = self.emit_value(operand, context)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value =
                    TypeConversion::from_target_type(operand_type, context).emit(value, context);
                let result = value.not(context);
                Ok(result)
            }
            Operator::Not => {
                let value = self.emit_value(operand, context)?;
                let zero = Value::constant(0, value.r#type(), context);
                let cmp = value.compare(zero, CmpPredicate::Eq, context);
                let result_type = target_type.unwrap_or_else(|| {
                    Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD)
                });
                let result =
                    TypeConversion::from_target_type(result_type, context).emit(cmp, context);
                Ok(result)
            }
            Operator::Subtract => {
                let magnitude = match operand {
                    Expression::DecimalNumberExpression(number) => number.integer_value(),
                    Expression::HexNumberExpression(number) => number.integer_value(),
                    _ => None,
                };
                if let (Some(operand_type), Some(magnitude)) = (target_type, magnitude) {
                    return Ok(Value::constant_from_bigint(
                        &-magnitude,
                        operand_type,
                        context,
                    ));
                }
                let value = self.emit_value(operand, context)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value =
                    TypeConversion::from_target_type(operand_type, context).emit(value, context);
                let zero = Value::constant(0, operand_type, context);
                let result = operator.emit(self.checked, zero, value, context);
                Ok(result)
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables via
    /// `resolve_to_definition()`.
    fn emit_increment_decrement(
        &self,
        operand: &Expression,
        operator: Operator,
        context: &Context<'context>,
    ) -> anyhow::Result<(Value<'context>, Value<'context>)> {
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported operand for {operator:?}");
        };
        let name = identifier.name();

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                let element_type =
                    TypeConversion::resolve_state_variable_type(&state_variable, context)?;
                let old = self.emit_storage_load(slot, element_type, context);
                let one = Value::constant(1, element_type, context);
                let new_value = operator.emit(self.checked, old, one, context);
                self.emit_storage_store(slot, new_value, element_type, context);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = pointer.load(element_type, context);
                let typed_one = Value::constant(1, element_type, context);
                let new_value = operator.emit(self.checked, old, typed_one, context);
                pointer.store(new_value, context);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
