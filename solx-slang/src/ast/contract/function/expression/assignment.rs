//!
//! Assignment expression lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

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
        assign: &slang_solidity_v2::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();
        let (target, block) = match &left {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let declared_type = state_variable.get_type().ok_or_else(|| {
                            anyhow::anyhow!("unresolved type for state variable: {name}")
                        })?;
                        if declared_type.is_reference_type() {
                            unimplemented!(
                                "assignment to a reference-typed state variable is not yet supported"
                            );
                        }
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .ok_or_else(|| {
                                anyhow::anyhow!("unregistered state variable: {name}")
                            })?;
                        let element_type = TypeConversion::resolve_slang_type(
                            &declared_type,
                            None,
                            &self.state.builder,
                        );
                        AssignmentTarget::Storage(*slot, element_type)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        AssignmentTarget::Pointer(pointer, element_type)
                    }
                    None => unreachable!("slang resolves every identifier reference"),
                    Some(_) => unimplemented!(
                        "assignment to non-variable definition '{name}' is not yet supported"
                    ),
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
            Expression::MemberAccessExpression(access) => {
                let (address, element_type, block) = self
                    .emit_struct_field_address(access, block)?
                    .expect("slang validates a member-access lvalue resolves to a struct field");
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed struct field in storage/calldata is not yet supported"
                    );
                }
                (AssignmentTarget::Pointer(address, element_type), block)
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(&left)
            ),
        };

        let right = assign.right_operand();
        let (value, block) = if matches!(
            assign.operator(),
            ast::AssignmentExpressionOperator::Equal(_)
        ) {
            self.emit_value(&right, block)?
        } else {
            let operator = match assign.operator() {
                ast::AssignmentExpressionOperator::AmpersandEqual(_) => Operator::BitwiseAnd,
                ast::AssignmentExpressionOperator::AsteriskEqual(_) => Operator::Multiply,
                ast::AssignmentExpressionOperator::BarEqual(_) => Operator::BitwiseOr,
                ast::AssignmentExpressionOperator::CaretEqual(_) => Operator::BitwiseXor,
                ast::AssignmentExpressionOperator::Equal(_) => {
                    unreachable!("should already be handled")
                }
                ast::AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_) => {
                    Operator::ShiftRight
                }
                ast::AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                    Operator::ShiftRight
                }
                ast::AssignmentExpressionOperator::LessThanLessThanEqual(_) => Operator::ShiftLeft,
                ast::AssignmentExpressionOperator::MinusEqual(_) => Operator::Subtract,
                ast::AssignmentExpressionOperator::PercentEqual(_) => Operator::Remainder,
                ast::AssignmentExpressionOperator::PlusEqual(_) => Operator::Add,
                ast::AssignmentExpressionOperator::SlashEqual(_) => Operator::Divide,
            };
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
