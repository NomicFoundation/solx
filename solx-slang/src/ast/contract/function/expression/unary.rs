//!
//! Unary expression lowering: prefix and postfix operators, increment/decrement.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits postfix `++` or `--` (returns the old value).
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
    /// When `target_type` is `Some`, unary operations use that type (matching
    /// solc's typed MLIR). When `None`, falls back to ui256 semantics.
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
                let value = TypeConversion::from_target_type(operand_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let result = block
                    .append_operation(
                        NotOperation::builder(
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                        )
                        .value(value)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.not always produces one result")
                    .into();
                Ok((result, block))
            }
            Operator::Not => {
                let (value, block) = self.emit_value(operand, block)?;
                let zero = self
                    .state
                    .builder
                    .emit_sol_constant(0, value.r#type(), &block);
                let cmp = self
                    .state
                    .builder
                    .emit_sol_cmp(value, zero, CmpPredicate::Eq, &block);
                let result_type = target_type.unwrap_or(self.state.builder.types.ui256);
                let result = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(cmp, &self.state.builder, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction. Checked negation
                // requires signed-type awareness (e.g. -INT_MIN should revert
                // in checked mode) which needs a dedicated op — not sol.csub,
                // since the operand may be in an unsigned literal type.
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let zero = self
                    .state
                    .builder
                    .emit_sol_constant(0, operand_type, &block);
                let result = block
                    .append_operation(
                        SubOperation::builder(
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                        )
                        .lhs(zero)
                        .rhs(value)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.sub always produces one result")
                    .into();
                Ok((result, block))
            }
            _ => unimplemented!("unsupported prefix operator: {operator:?}"),
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
            unimplemented!("unsupported operand for {operator:?}");
        };

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .unwrap_or_else(|| {
                        unimplemented!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                )?;
                let old = self.emit_storage_load(slot, element_type, block)?;
                let new_value = self.emit_step(old, operator, element_type, block);
                self.emit_storage_store(slot, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                let (pointer, element_type) =
                    self.environment.variable_with_type(definition.node_id());
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let new_value = self.emit_step(old, operator, element_type, block);
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => unreachable!("slang resolves every identifier reference"),
            Some(other) => {
                unimplemented!(
                    "unsupported operand for {operator:?}: {:?}",
                    other.node_id()
                )
            }
        }
    }

    /// Applies the `++` / `--` step to a loaded value: adds or subtracts a typed
    /// `1` through the operator's binary operation, honoring the arithmetic
    /// mode. The two lvalue kinds (storage slot and stack pointer) share this
    /// step; only the surrounding load/store differs.
    fn emit_step(
        &self,
        old: Value<'context, 'block>,
        operator: Operator,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let one = self.state.builder.emit_sol_constant(1, element_type, block);
        block
            .append_operation(operator.emit_sol_binary_operation(
                self.arithmetic_mode,
                self.state.builder.context,
                self.state.builder.unknown_location,
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
    }
}
