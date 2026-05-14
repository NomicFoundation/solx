//!
//! Assignment expression lowering.
//!

use std::str::FromStr;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(u64, Type<'context>),
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    pub fn emit_assignment(
        &self,
        assign: &slang_solidity::backend::ir::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let (target, block) = match &left {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .ok_or_else(|| {
                                anyhow::anyhow!("unregistered state variable: {name}")
                            })?;
                        let element_type = TypeConversion::resolve_state_variable_type(
                            &state_variable,
                            &self.state.builder,
                        )?;
                        AssignmentTarget::Storage(*slot, element_type)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        AssignmentTarget::Pointer(pointer, element_type)
                    }
                    None => anyhow::bail!("unresolved identifier: {name}"),
                    Some(_) => anyhow::bail!("unsupported assignment target: {name}"),
                };
                (target, block)
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed `a[i]` element in storage/calldata is not yet supported"
                    );
                }
                (AssignmentTarget::Pointer(address, element_type), block)
            }
            _ => anyhow::bail!("unsupported assignment target"),
        };

        let operator = assign.operator();
        let operator_text = operator.text.as_str();
        let right = assign.right_operand();
        let (value, block) = if operator_text == "=" {
            self.emit_value(&right, block)?
        } else {
            let (old, target_type) = match target {
                AssignmentTarget::Pointer(pointer, element_type) => {
                    let old = self
                        .state
                        .builder
                        .emit_sol_load(pointer, element_type, &block)?;
                    (old, element_type)
                }
                AssignmentTarget::Storage(slot, element_type) => {
                    let old = self.emit_storage_load(slot, element_type, &block)?;
                    (old, element_type)
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
            AssignmentTarget::Pointer(pointer, element_type) => {
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
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, &block);
                self.emit_storage_store(slot, stored_value, &block);
                stored_value
            }
        };
        Ok((result, block))
    }
}
