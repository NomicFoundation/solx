//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use std::str::FromStr;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::SubOperation;
use solx_mlir::ods::sol::XorOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a binary arithmetic Sol dialect operation.
    ///
    /// When `target_type` is `Some`, both operands are cast to that type and
    /// the result has that type (matching solc's type-annotated MLIR output).
    /// When `None`, selects the wider operand type by bit width.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = solx_mlir::TypeFactory::integer_bit_width(lhs.r#type());
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let operator = Operator::from_str(operator)?;
        let value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.builder.context,
                self.state.builder.unknown_location,
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: &str,
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
        operator: &str,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match operator {
            "++" | "--" => {
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            "~" => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let all_ones = self
                    .state
                    .builder
                    .emit_sol_constant_all_ones(operand_type, &block);
                let result = block
                    .append_operation(
                        XorOperation::builder(
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                        )
                        .lhs(value)
                        .rhs(all_ones)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.xor always produces one result")
                    .into();
                Ok((result, block))
            }
            "!" => {
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
            "-" => {
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
            _ => anyhow::bail!("unsupported prefix operator: {operator}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables via
    /// `resolve_to_definition()`.
    fn emit_increment_decrement(
        &self,
        operand: &Expression,
        operator: &str,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported operand for {operator}");
        };
        let name = identifier.name();
        let operator = Operator::from_str(operator)?;

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = *self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                )?;
                let old = self.emit_storage_load(slot, element_type, block)?;
                let one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.emit_storage_store(slot, new_value, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self
                    .environment
                    .variable_with_type(&name)
                    .ok_or_else(|| anyhow::anyhow!("unregistered local variable: {name}"))?;
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let typed_one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        typed_one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
