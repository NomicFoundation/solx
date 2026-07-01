//!
//! Assignment expression lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_place::EmitPlace;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for AssignmentExpression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let left = self.left_operand();
        let (target, block) = match &left {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let declared_type = state_variable
                            .get_type()
                            .expect("binder types every state variable");
                        if declared_type.is_reference_type() {
                            unimplemented!(
                                "assignment to a reference-typed state variable is not yet supported"
                            );
                        }
                        let slot = context
                            .storage_layout
                            .get(&state_variable.node_id())
                            .expect("state variable is registered in the storage layout")
                            .clone();
                        let element_type = TypeConversion::resolve_slang_type(
                            &declared_type,
                            None,
                            context.state,
                        );
                        AssignmentTarget::Storage(slot, element_type)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) =
                            context.environment.variable_with_type(&name);
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
                let BlockAnd {
                    value: place,
                    block,
                } = index_access.emit_place(context, block);
                if place.address.r#type() == place.element_type {
                    unimplemented!(
                        "assignment to a reference-typed `a[i]` element in storage/calldata is not yet supported"
                    );
                }
                (
                    AssignmentTarget::Pointer(place.address, place.element_type),
                    block,
                )
            }
            Expression::MemberAccessExpression(access) => {
                let BlockAnd {
                    value: place,
                    block,
                } = access.emit_place(context, block);
                if place.address.r#type() == place.element_type {
                    unimplemented!(
                        "assignment to a reference-typed struct field in storage/calldata is not yet supported"
                    );
                }
                (
                    AssignmentTarget::Pointer(place.address, place.element_type),
                    block,
                )
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(&left)
            ),
        };

        let right = self.right_operand();
        let (value, block) = if matches!(
            self.operator(),
            ast::AssignmentExpressionOperator::Equal(_)
        ) {
            let BlockAnd { value, block } = right.emit(context, block);
            (value, block)
        } else {
            let operator = match self.operator() {
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
                AssignmentTarget::Pointer(pointer, element_type) => {
                    let old = Pointer::new(*pointer)
                        .load(AstType::new(*element_type), context.state, &block)
                        .into_mlir();
                    (old, *element_type)
                }
                AssignmentTarget::Storage(slot, element_type) => {
                    let old = context.emit_storage_load(slot, *element_type, &block);
                    (old, *element_type)
                }
            };
            let BlockAnd { value: rhs, block } = right.emit(context, block);
            let old = TypeConversion::from_target_type(target_type, context.state)
                .emit(old, context.state, &block);
            let rhs = if matches!(operator, Operator::ShiftLeft | Operator::ShiftRight) {
                rhs
            } else {
                TypeConversion::from_target_type(target_type, context.state)
                    .emit(rhs, context.state, &block)
            };
            let result = block
                .append_operation(operator.emit_sol_binary_operation(
                    context.checked,
                    context.state.mlir_context,
                    context.state.location(),
                    old,
                    rhs,
                ))
                .result(0)
                .expect("binary operation always produces one result")
                .into();
            (result, block)
        };

        let value = match &target {
            AssignmentTarget::Pointer(pointer, element_type) => {
                let stored_value = TypeConversion::from_target_type(*element_type, context.state)
                    .emit(value, context.state, &block);
                Pointer::new(*pointer).store(AstValue::new(stored_value), context.state, &block);
                stored_value
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value = TypeConversion::from_target_type(*element_type, context.state)
                    .emit(value, context.state, &block);
                context.emit_storage_store(slot, stored_value, *element_type, &block);
                stored_value
            }
        };
        BlockAnd { block, value }
    }
}
