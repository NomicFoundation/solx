//!
//! Assignment expression lowering.
//!

use std::str::FromStr;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Local variable — alloca'd pointer and its declared element type.
    Local(Value<'context, 'block>, Type<'context>),
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
                let (pointer, element_type) = self
                    .environment
                    .variable_with_type(&name)
                    .ok_or_else(|| anyhow::anyhow!("unregistered local variable: {name}"))?;
                AssignmentTarget::Local(pointer, element_type)
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported assignment target: {name}"),
        };

        let operator = assign.operator();
        let operator_text = operator.text.as_str();
        let right = assign.right_operand();
        let (value, block) = if operator_text == "=" {
            self.emit_value(&right, block)?
        } else {
            let (old, target_type) = match target {
                AssignmentTarget::Local(pointer, element_type) => {
                    let old = self
                        .state
                        .builder
                        .emit_sol_load(pointer, element_type, &block)?;
                    (old, element_type)
                }
                AssignmentTarget::Storage(slot) => {
                    let old = self.emit_storage_load(slot, &block)?;
                    (old, self.state.builder.get_type(solx_mlir::Builder::UI256))
                }
            };
            let (rhs, block) = self.emit_value(&right, block)?;
            let old = TypeConversion::from_target_type(target_type, &self.state.builder).emit(
                old,
                &self.state.builder,
                &block,
            );
            let rhs = TypeConversion::from_target_type(target_type, &self.state.builder).emit(
                rhs,
                &self.state.builder,
                &block,
            );
            let operator = Operator::from_str(operator_text)?.arithmetic_operator();
            let result = block
                .append_operation(operator.emit_sol_binary_operation(
                    self.checked,
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                    old,
                    rhs,
                ))
                .result(0)
                .expect("binary operation always produces one result")
                .into();
            (result, block)
        };

        let result = match target {
            AssignmentTarget::Local(pointer, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, &block);
                self.state
                    .builder
                    .emit_sol_store(stored_value, pointer, &block);
                stored_value
            }
            AssignmentTarget::Storage(slot) => {
                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let stored_value = self.state.builder.emit_sol_cast(value, ui256, &block);
                self.emit_storage_store(slot, stored_value, &block);
                value
            }
        };
        Ok((result, block))
    }
}
