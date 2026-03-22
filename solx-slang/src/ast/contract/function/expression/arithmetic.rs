//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use std::str::FromStr;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::ICmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a binary arithmetic LLVM operation.
    ///
    /// Uses signed LLVM operations (`sdiv`, `srem`, `ashr`) when either
    /// operand has a signed integer type.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let signed = Self::is_signed(left) || Self::is_signed(right);
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        // TODO: use sol.cadd/csub/cmul for checked arithmetic (Solidity 0.8+ default)
        let operator = Operator::from_str(operator)?;
        let value =
            self.emit_llvm_operation(operator.llvm_operation_name(signed), lhs, rhs, &block)?;
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
        let _ = new_value;
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
                let (value, block) = self.emit(operand, block)?;
                let all_ones = self.state.builder().emit_sol_constant(-1, &block);
                let result =
                    self.emit_llvm_operation(solx_mlir::Builder::XOR, value, all_ones, &block)?;
                Ok((result, block))
            }
            "!" => {
                let (value, block) = self.emit(operand, block)?;
                let zero = self.state.builder().emit_sol_constant(0, &block);
                let cmp = self
                    .state
                    .builder()
                    .emit_icmp(value, zero, ICmpPredicate::Eq, &block);
                let result = self.state.builder().emit_zext_to_i256(cmp, &block);
                Ok((result, block))
            }
            "-" => {
                let (value, block) = self.emit(operand, block)?;
                let zero = self.state.builder().emit_sol_constant(0, &block);
                let result =
                    self.emit_llvm_operation(solx_mlir::Builder::SUB, zero, value, &block)?;
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
        let one = self.state.builder().emit_sol_constant(1, block);

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = *self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                let old = self.emit_storage_load(slot, block)?;
                let new_value =
                    self.emit_llvm_operation(operator.llvm_operation_name(false), old, one, block)?;
                self.emit_storage_store(slot, new_value, block)?;
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let pointer = self
                    .environment
                    .variable(&name)
                    .ok_or_else(|| anyhow::anyhow!("unregistered local variable: {name}"))?;
                let old = self.emit_load(pointer, block)?;
                let new_value =
                    self.emit_llvm_operation(operator.llvm_operation_name(false), old, one, block)?;
                self.emit_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
