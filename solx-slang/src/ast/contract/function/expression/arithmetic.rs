//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use std::str::FromStr;

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::CmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// All-ones `ui256` value (`2^256 - 1`) for bitwise NOT.
    const UI256_ALL_ONES: &'static str =
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

    /// Emits a binary arithmetic Sol dialect operation.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;
        let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
        let lhs = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let operator = Operator::from_str(operator)?;
        let value = self.state.builder.emit_binary_operation(
            operator.sol_operation_name(),
            lhs,
            rhs,
            ui256,
            &block,
        )?;
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
    pub fn emit_prefix(
        &self,
        operator: &str,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match operator {
            "++" | "--" => {
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            "~" => {
                let (value, block) = self.emit_value(operand, block)?;
                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let value = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                );
                let all_ones = self.state.builder.emit_sol_constant_from_hex_str(
                    Self::UI256_ALL_ONES,
                    ui256,
                    &block,
                )?;
                let result = self.state.builder.emit_binary_operation(
                    solx_mlir::Builder::SOL_XOR,
                    value,
                    all_ones,
                    ui256,
                    &block,
                )?;
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
                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let result = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
                    cmp,
                    &self.state.builder,
                    &block,
                );
                Ok((result, block))
            }
            "-" => {
                let (value, block) = self.emit_value(operand, block)?;
                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let value = TypeConversion::from_target_type(ui256, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                );
                let zero = self.state.builder.emit_sol_constant(0, ui256, &block);
                let result = self.state.builder.emit_binary_operation(
                    solx_mlir::Builder::SOL_SUB,
                    zero,
                    value,
                    ui256,
                    &block,
                )?;
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
                let old = self.emit_storage_load(slot, block)?;
                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let one = self.state.builder.emit_sol_constant(1, ui256, block);
                let new_value = self.state.builder.emit_binary_operation(
                    operator.sol_operation_name(),
                    old,
                    one,
                    ui256,
                    block,
                )?;
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
                let one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = self.state.builder.emit_binary_operation(
                    operator.sol_operation_name(),
                    old,
                    one,
                    element_type,
                    block,
                )?;
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
