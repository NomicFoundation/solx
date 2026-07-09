//!
//! Assignment expression lowering.
//!

use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Place(Place<'context>, Type<'context>),
    /// State variable: storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
}

impl<'state, 'context> ExpressionEmitter<'state, 'context> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    pub fn emit_assignment(
        &self,
        assign: &slang_solidity_v2::ast::AssignmentExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let left = assign.left_operand();
        let target = match &left {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
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
                            .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?
                            .clone();
                        let element_type =
                            TypeConversion::resolve_slang_type(&declared_type, None, context);
                        AssignmentTarget::Storage(slot, element_type)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        AssignmentTarget::Place(pointer, element_type)
                    }
                    None => unreachable!("slang resolves every identifier reference"),
                    Some(_) => unimplemented!(
                        "assignment to non-variable definition '{name}' is not yet supported"
                    ),
                }
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type) =
                    self.emit_index_access_address(index_access, context)?;
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed `a[i]` element in storage/calldata is not yet supported"
                    );
                }
                AssignmentTarget::Place(address, element_type)
            }
            Expression::MemberAccessExpression(access) => {
                let (address, element_type) = self
                    .emit_struct_field_address(access, context)?
                    .expect("slang validates a member-access lvalue resolves to a struct field");
                if address.r#type() == element_type {
                    unimplemented!(
                        "assignment to a reference-typed struct field in storage/calldata is not yet supported"
                    );
                }
                AssignmentTarget::Place(address, element_type)
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(&left)
            ),
        };

        let right = assign.right_operand();
        let value = if matches!(
            assign.operator(),
            ast::AssignmentExpressionOperator::Equal(_)
        ) {
            self.emit_value(&right, context)?
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
            let (old, target_type) = match &target {
                AssignmentTarget::Place(pointer, element_type) => {
                    let old = pointer.load(*element_type, context);
                    (old, *element_type)
                }
                AssignmentTarget::Storage(slot, element_type) => {
                    let old = self.emit_storage_load(slot, *element_type, context);
                    (old, *element_type)
                }
            };
            let rhs = self.emit_value(&right, context)?;
            let old = TypeConversion::from_target_type(target_type, context).emit(old, context);
            let rhs = TypeConversion::from_target_type(target_type, context).emit(rhs, context);
            operator.emit(self.checked, old, rhs, context)
        };

        let result = match &target {
            AssignmentTarget::Place(pointer, element_type) => {
                let stored_value =
                    TypeConversion::from_target_type(*element_type, context).emit(value, context);
                pointer.store(stored_value, context);
                stored_value
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value =
                    TypeConversion::from_target_type(*element_type, context).emit(value, context);
                self.emit_storage_store(slot, stored_value, *element_type, context);
                stored_value
            }
        };
        Ok(result)
    }
}
