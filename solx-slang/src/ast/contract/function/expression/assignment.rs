//!
//! Assignment expression emission.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitPlace;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::contract::storage_layout::StorageSlot;

/// Assignment target resolved from the Slang binder.
pub enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its element type — a local, parameter, or `a[i]` / `m[k]` lvalue.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
}

impl<'context, 'block> AssignmentTarget<'context, 'block> {
    /// Resolves a single left-hand-side expression to its assignment target.
    fn new<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        target_expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        match target_expression {
            Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => {
                    Self::from_state_variable(context, &state_variable, block)
                }
                Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                    let pointer = Pointer::new(context.environment.variable(definition.node_id()));
                    (
                        Self::Pointer(pointer.into_mlir(), pointer.pointee().into_mlir()),
                        block,
                    )
                }
                None => unreachable!("slang resolves every identifier reference"),
                Some(other) => unimplemented!(
                    "assignment to non-variable definition {:?} is not yet supported",
                    other.node_id()
                ),
            },
            Expression::IndexAccessExpression(index_access) => {
                let BlockAnd {
                    value:
                        Place {
                            address,
                            element_type,
                        },
                    block,
                } = index_access.emit_place(context, block);
                (Self::from_address(address, element_type), block)
            }
            Expression::MemberAccessExpression(access) => {
                let BlockAnd {
                    value:
                        Place {
                            address,
                            element_type,
                        },
                    block,
                } = access.emit_place(context, block);
                (Self::from_address(address, element_type), block)
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(target_expression)
            ),
        }
    }

    /// Resolves a state-variable lvalue (bare `x`) to its target. A reference-typed
    /// state variable is not yet supported; a value-typed one stores the scalar directly.
    fn from_state_variable<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        let declared_type = state_variable.get_type().expect("slang validated");
        if declared_type.is_reference_type() {
            unimplemented!("assignment to a reference-typed state variable is not yet supported");
        }
        let slot = context
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            })
            .clone();
        let element_type = AstType::resolve(
            &declared_type,
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        (Self::Storage(slot, element_type), block)
    }

    /// Classifies a computed lvalue `address` into its target. A reference element (the address type
    /// IS the element type) is not yet supported; any other becomes a [`Self::Pointer`].
    fn from_address(address: Value<'context, 'block>, element_type: Type<'context>) -> Self {
        if address.r#type() == element_type {
            unimplemented!(
                "assignment to a reference-typed lvalue in storage/calldata is not yet supported"
            );
        }
        Self::Pointer(address, element_type)
    }

    /// Stores a coerced value into this target.
    fn store<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match self {
            Self::Pointer(pointer, element_type) => {
                let stored_value = AstValue::from(value).cast(
                    AstType::new(*element_type),
                    &context.state.builder,
                    block,
                );
                Pointer::new(*pointer).store(stored_value, &context.state.builder, block);
                stored_value.into_mlir()
            }
            Self::Storage(slot, element_type) => {
                let stored_value = AstValue::from(value)
                    .cast(AstType::new(*element_type), &context.state.builder, block)
                    .into_mlir();
                slot.store(&context.state.builder, stored_value, *element_type, block);
                stored_value
            }
        }
    }
}

// An assignment expression (`=`, `+=`, `-=`, `*=`, …).
expression_emit!(AssignmentExpression; |node, context, block| {
    let left = node.left_operand();
    let right = node.right_operand();

    let (target, block) = AssignmentTarget::new(context, &left, block);
    let (value, block) = if matches!(node.operator(), ast::AssignmentExpressionOperator::Equal(_)) {
        let BlockAnd { value, block } = right.emit(context, block);
        (value, block)
    } else {
        let operator = match node.operator() {
            ast::AssignmentExpressionOperator::AmpersandEqual(_) => Operator::BitwiseAnd,
            ast::AssignmentExpressionOperator::AsteriskEqual(_) => Operator::Multiply,
            ast::AssignmentExpressionOperator::BarEqual(_) => Operator::BitwiseOr,
            ast::AssignmentExpressionOperator::CaretEqual(_) => Operator::BitwiseXor,
            ast::AssignmentExpressionOperator::Equal(_) => {
                unreachable!("should already be handled")
            }
            ast::AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_)
            | ast::AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
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
                let old = Pointer::new(*pointer).load(
                    AstType::new(*element_type),
                    &context.state.builder,
                    &block,
                );
                (old, *element_type)
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let old = slot.load(&context.state.builder, *element_type, &block);
                (AstValue::from(old), *element_type)
            }
        };
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        let result = operator.emit_value_binary(
            context.arithmetic_mode,
            &context.state.builder,
            old,
            rhs,
            target_type,
            &block,
        );
        (result, block)
    };

    let result = target.store(context, value.into_mlir(), &block);
    BlockAnd { block, value: result.into() }
});
