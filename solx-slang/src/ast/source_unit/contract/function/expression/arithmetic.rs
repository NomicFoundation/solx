//!
//! Arithmetic expression lowering: binary ops, prefix, postfix, assignment.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::ICmpPredicate;

use crate::ast::source_unit::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    pub(super) fn emit_assignment(
        &self,
        assign: &slang_solidity::backend::ir::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let Expression::Identifier(identifier) = &left else {
            anyhow::bail!("unsupported assignment target");
        };
        let name = identifier.name();

        // Determine whether this is a local variable or a state variable.
        let local_pointer = self.environment.variable(&name);
        let storage_slot = self.state.state_variable_slot(&name);
        if local_pointer.is_none() && storage_slot.is_none() {
            anyhow::bail!("undefined variable: {name}");
        }

        let operator = assign.operator();
        let operator_text = operator.text.as_str();
        let right = assign.right_operand();
        let (value, block) = if operator_text == "=" {
            self.emit(&right, block)?
        } else {
            let old = if let Some(pointer) = local_pointer {
                self.emit_load(pointer, &block)?
            } else {
                let slot = storage_slot.ok_or_else(|| {
                    anyhow::anyhow!("state variable '{name}' has no assigned storage slot")
                })?;
                self.emit_storage_load(slot, &block)?
            };
            let (rhs, block) = self.emit(&right, block)?;
            let arithmetic_operation = match operator_text {
                "+=" => solx_mlir::ops::ADD,
                "-=" => solx_mlir::ops::SUB,
                "*=" => solx_mlir::ops::MUL,
                "/=" => solx_mlir::ops::UDIV,
                "%=" => solx_mlir::ops::UREM,
                _ => anyhow::bail!("unsupported assignment operator: {operator_text}"),
            };
            let result = self.emit_llvm_operation(arithmetic_operation, old, rhs, &block)?;
            (result, block)
        };

        if let Some(pointer) = local_pointer {
            self.emit_store(value, pointer, &block);
        } else {
            let slot = storage_slot.ok_or_else(|| {
                anyhow::anyhow!("state variable '{name}' has no assigned storage slot")
            })?;
            self.emit_storage_store(slot, value, &block);
        }
        Ok((value, block))
    }

    /// Emits a binary arithmetic LLVM operation.
    pub(super) fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        let operation_name = match operator {
            "+" => solx_mlir::ops::ADD,
            "-" => solx_mlir::ops::SUB,
            "*" => solx_mlir::ops::MUL,
            "/" => solx_mlir::ops::UDIV,
            "%" => solx_mlir::ops::UREM,
            "&" => solx_mlir::ops::AND,
            "|" => solx_mlir::ops::OR,
            "^" => solx_mlir::ops::XOR,
            "<<" => solx_mlir::ops::SHL,
            ">>" => solx_mlir::ops::LSHR,
            _ => anyhow::bail!("unsupported binary operator: {operator}"),
        };
        let value = self.emit_llvm_operation(operation_name, lhs, rhs, &block)?;
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub(super) fn emit_postfix(
        &self,
        operand: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported postfix operand");
        };
        let name = identifier.name();
        let pointer = self
            .environment
            .variable(&name)
            .ok_or_else(|| anyhow::anyhow!("undefined variable: {name}"))?;
        let old = self.emit_load(pointer, &block)?;
        let one = self.state.emit_sol_constant(1, &block);
        let operation_name = match operator {
            "++" => solx_mlir::ops::ADD,
            "--" => solx_mlir::ops::SUB,
            _ => anyhow::bail!("unsupported postfix operator: {operator}"),
        };
        let new = self.emit_llvm_operation(operation_name, old, one, &block)?;
        self.emit_store(new, pointer, &block);
        Ok((old, block))
    }

    /// Emits prefix `!` or `-`.
    pub(super) fn emit_prefix(
        &self,
        operator: &str,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(operand, block)?;
        match operator {
            "!" => {
                let zero = self.state.emit_sol_constant(0, &block);
                let cmp = self.state.emit_icmp(value, zero, ICmpPredicate::Eq, &block);
                let result = self.state.emit_zext_to_i256(cmp, &block);
                Ok((result, block))
            }
            "-" => {
                let zero = self.state.emit_sol_constant(0, &block);
                let result = self.emit_llvm_operation(solx_mlir::ops::SUB, zero, value, &block)?;
                Ok((result, block))
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator}"),
        }
    }
}
