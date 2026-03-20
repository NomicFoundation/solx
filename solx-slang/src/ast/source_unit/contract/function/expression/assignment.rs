//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use crate::ast::source_unit::contract::function::expression::ExpressionEmitter;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Local variable — alloca'd pointer.
    Local(Value<'context, 'block>),
    /// State variable — storage slot.
    Storage(u64),
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    pub fn emit_assignment(
        &self,
        assign: &slang_solidity::backend::ir::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let Expression::Identifier(identifier) = &left else {
            anyhow::bail!("unsupported assignment target");
        };
        let name = identifier.name();

        let target = match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                AssignmentTarget::Storage(*slot)
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let pointer = self
                    .environment
                    .variable(&name)
                    .ok_or_else(|| anyhow::anyhow!("unregistered local variable: {name}"))?;
                AssignmentTarget::Local(pointer)
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported assignment target: {name}"),
        };

        let signed = Self::is_signed(&left);
        let operator = assign.operator();
        let operator_text = operator.text.as_str();
        let right = assign.right_operand();
        let (value, block) = if operator_text == "=" {
            self.emit(&right, block)?
        } else {
            let old = match target {
                AssignmentTarget::Local(pointer) => self.emit_load(pointer, &block)?,
                AssignmentTarget::Storage(slot) => self.emit_storage_load(slot, &block)?,
            };
            let (rhs, block) = self.emit(&right, block)?;
            // TODO: change to a nice enum with FromStr
            let arithmetic_operation = match operator_text {
                "+=" => solx_mlir::ops::ADD,
                "-=" => solx_mlir::ops::SUB,
                "*=" => solx_mlir::ops::MUL,
                "/=" if signed => solx_mlir::ops::SDIV,
                "/=" => solx_mlir::ops::UDIV,
                "%=" if signed => solx_mlir::ops::SREM,
                "%=" => solx_mlir::ops::UREM,
                _ => anyhow::bail!("unsupported assignment operator: {operator_text}"),
            };
            let result = self.emit_llvm_operation(arithmetic_operation, old, rhs, &block)?;
            (result, block)
        };

        match target {
            AssignmentTarget::Local(pointer) => self.emit_store(value, pointer, &block),
            AssignmentTarget::Storage(slot) => {
                self.emit_storage_store(slot, value, &block)?;
            }
        }
        Ok((value, block))
    }
}
