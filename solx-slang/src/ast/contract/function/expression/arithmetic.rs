//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = lhs.r#type().integer_bit_width();
            let rhs_width = rhs.r#type().integer_bit_width();
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs =
            TypeConversion::from_target_type(result_type, self.state).emit(lhs, self.state, &block);
        let rhs =
            TypeConversion::from_target_type(result_type, self.state).emit(rhs, self.state, &block);
        let value = operator.emit(self.checked, lhs, rhs, self.state, &block);
        Ok((value, block))
    }

    /// Emits postfix `++` or `--`, returning the old value.
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (old, _) = self.emit_increment_decrement(operand, operator, &block)?;
        Ok((old, block))
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match operator {
            Operator::Increment | Operator::Decrement => {
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            Operator::BitwiseNot => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let result = value.not(self.state, &block);
                Ok((result, block))
            }
            Operator::Not => {
                let (value, block) = self.emit_value(operand, block)?;
                let zero = Value::constant(0, value.r#type(), self.state, &block);
                let cmp = value.compare(zero, CmpPredicate::Eq, self.state, &block);
                let result_type = target_type.unwrap_or_else(|| {
                    Type::unsigned(self.state.melior, solx_utils::BIT_LENGTH_FIELD)
                });
                let result = TypeConversion::from_target_type(result_type, self.state)
                    .emit(cmp, self.state, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let zero = Value::constant(0, operand_type, self.state, &block);
                let result = zero.subtract(value, false, self.state, &block);
                Ok((result, block))
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
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
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
                    TypeConversion::resolve_state_variable_type(&state_variable, self.state)?;
                let old = self.emit_storage_load(slot, element_type, block);
                let one = Value::constant(1, element_type, self.state, block);
                let new_value = operator.emit(self.checked, old, one, self.state, block);
                self.emit_storage_store(slot, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = pointer.load(element_type, self.state, block);
                let typed_one = Value::constant(1, element_type, self.state, block);
                let new_value = operator.emit(self.checked, old, typed_one, self.state, block);
                pointer.store(new_value, self.state, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
