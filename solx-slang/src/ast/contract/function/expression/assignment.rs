//!
//! Assignment expression emission.
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
use slang_solidity_v2::ast::TupleExpression;
use solx_mlir::ods::sol::DeleteOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
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
    /// Reference-typed location: the destination into which the RHS reference's contents are copied via `sol.copy`.
    ReferenceCopy(Value<'context, 'block>),
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
                Some(other) => unreachable!(
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
                // A namespace-qualified state-variable lvalue (`C.x = v`) is not a struct field;
                // resolve it like the bare `x = v`. A genuine struct field falls through below.
                if let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                {
                    return Self::from_state_variable(context, &state_variable, block);
                }
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
            Expression::FunctionCallExpression(call)
                if matches!(
                    &call.operand(),
                    Expression::MemberAccessExpression(access)
                        if matches!(
                            access.member().resolve_to_built_in(),
                            Some(ast::BuiltIn::ArrayPush)
                        )
                ) =>
            {
                // `arr.push() = v` — `push` appends a default element and returns a
                // reference to it; that reference is the lvalue (like `arr.push(v)`).
                let Expression::MemberAccessExpression(access) = call.operand() else {
                    unreachable!("guarded by the match arm");
                };
                let base_slang_type = access.operand().get_type().expect("slang validated");
                let BlockAnd {
                    value: array_value,
                    block,
                } = access.operand().emit(context, block);
                let (new_slot, element_type) =
                    array_value.push_slot(&base_slang_type, &context.state.builder, &block);
                (
                    Self::from_address(new_slot.into_mlir(), element_type),
                    block,
                )
            }
            _ => unreachable!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(target_expression)
            ),
        }
    }

    /// Resolves a state-variable lvalue (bare `x` or `C.x`) to its target. Reference-typed storage
    /// is copied via `sol.copy` ([`Self::ReferenceCopy`]); value-typed storage stores the scalar directly.
    fn from_state_variable<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        let declared_type = state_variable.get_type().expect("slang validated");
        let slot = context
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unreachable!("unregistered state variable {:?}", state_variable.node_id())
            })
            .clone();
        let element_type = AstType::resolve(
            &declared_type,
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        if declared_type.is_reference_type() && !matches!(declared_type, ast::Type::Mapping(_)) {
            let address_type = AstType::new(element_type)
                .address_type(slot.location, context.state.builder.context);
            let storage_ref =
                Pointer::addr_of(&slot.name, address_type, &context.state.builder, &block)
                    .into_mlir();
            return (Self::ReferenceCopy(storage_ref), block);
        }
        (Self::Storage(slot, element_type), block)
    }

    /// Classifies a computed lvalue `address` into its target: a reference element (the address type
    /// IS the element type) becomes a [`Self::ReferenceCopy`], any other a [`Self::Pointer`].
    fn from_address(address: Value<'context, 'block>, element_type: Type<'context>) -> Self {
        if address.r#type() == element_type {
            Self::ReferenceCopy(address)
        } else {
            Self::Pointer(address, element_type)
        }
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
            Self::ReferenceCopy(address) => {
                // The RHS is already a reference of the matching type; copy its
                // contents into the destination reference (no scalar coercion).
                Pointer::new(*address).copy_from(
                    AstValue::from(value),
                    &context.state.builder,
                    block,
                );
                value
            }
        }
    }

    /// Collects the `(lvalue, value)` bindings of a destructuring assignment `(a, b, …) = rhs`,
    /// evaluating every value before any store (so `(a, b) = (b, a)` swaps). A blank slot discards its RHS.
    fn destructure<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        lhs: &TupleExpression,
        rhs: &Expression,
        mut block: BlockRef<'context, 'block>,
    ) -> (
        Vec<(Expression, Value<'context, 'block>)>,
        BlockRef<'context, 'block>,
    ) {
        let mut bindings = Vec::new();
        match rhs {
            Expression::TupleExpression(rhs) => {
                for (lvalue, rhs) in lhs.items().iter().zip(rhs.items().iter()) {
                    let rhs = rhs.expression().expect("slang validated");
                    match (lvalue.expression(), &rhs) {
                        (
                            Some(Expression::TupleExpression(lvalue)),
                            Expression::TupleExpression(_),
                        ) => {
                            let (nested, next) = Self::destructure(context, &lvalue, &rhs, block);
                            bindings.extend(nested);
                            block = next;
                        }
                        (Some(lvalue), _) => {
                            let BlockAnd { value, block: next } = rhs.emit(context, block);
                            bindings.push((lvalue, value.into_mlir()));
                            block = next;
                        }
                        (None, Expression::TupleExpression(_)) => {}
                        (None, _) => block = rhs.emit(context, block).block,
                    }
                }
            }
            _ => {
                let BlockAnd {
                    value: values,
                    block: next,
                } = match rhs {
                    Expression::FunctionCallExpression(call) => call.emit(context, block),
                    Expression::ConditionalExpression(conditional) => {
                        conditional.emit(context, block)
                    }
                    _ => unreachable!(
                        "tuple assignment with this right-hand side shape is not yet supported"
                    ),
                };
                block = next;
                for (lvalue, value) in lhs.items().iter().zip(values) {
                    if let Some(lvalue) = lvalue.expression() {
                        bindings.push((lvalue, value));
                    }
                }
            }
        }
        (bindings, block)
    }

    /// Resolves each lvalue left-to-right against the pre-assignment state, then stores RIGHT-TO-LEFT
    /// so the leftmost write to an aliased destination wins. Returns the last stored value, or zero if all blank.
    fn store_all<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        bindings: Vec<(Expression, Value<'context, 'block>)>,
        mut block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let mut targets = Vec::with_capacity(bindings.len());
        for (lvalue, value) in bindings {
            let (target, next) = Self::new(context, &lvalue, block);
            block = next;
            targets.push((target, value));
        }
        let result = targets
            .into_iter()
            .rev()
            .fold(None, |_, (target, value)| {
                Some(target.store(context, value, &block))
            })
            .unwrap_or_else(|| AstValue::uint256(0, &context.state.builder, &block).into_mlir());
        (result, block)
    }

    /// Emits `delete x` — resets the lvalue to its zero. A storage aggregate is deep-cleared via
    /// `sol.delete`; a memory aggregate resets to a zero-filled buffer; a value lvalue to its typed zero.
    pub fn delete<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let (target, block) = Self::new(context, operand, block);
        match &target {
            Self::ReferenceCopy(reference) => {
                mlir_op_void!(
                    &context.state.builder,
                    &block,
                    DeleteOperation.reference(*reference)
                );
            }
            Self::Pointer(_, element_type) | Self::Storage(_, element_type) => {
                let slang_type = operand.get_type().expect("slang validated");
                let zero = if slang_type.is_reference_type() {
                    // A memory aggregate resets to a freshly allocated zero-filled buffer, exactly as
                    // it is default-initialised — not a scalar zero.
                    match &slang_type {
                        ast::Type::String(_) | ast::Type::Bytes(_) => {
                            let size = AstValue::constant(
                                0,
                                AstType::unsigned(
                                    context.state.builder.context,
                                    solx_utils::BIT_LENGTH_FIELD,
                                ),
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir();
                            AstValue::malloc(
                                *element_type,
                                Some(size),
                                true,
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir()
                        }
                        _ => AstValue::malloc(
                            *element_type,
                            None,
                            true,
                            &context.state.builder,
                            &block,
                        )
                        .into_mlir(),
                    }
                } else {
                    // The zero of a value lvalue is its type's own zero, not a raw `ui256` 0 — emitting
                    // a `ui256` 0 and letting the store coerce it would be an ill-typed cast (e.g. to `func_ref`).
                    AstValue::zero(AstType::new(*element_type), &context.state.builder, &block)
                        .into_mlir()
                };
                target.store(context, zero, &block);
            }
        }
        block
    }
}

// An assignment expression (`=`, `+=`, `-=`, `*=`, …).
expression_emit!(AssignmentExpression; |node, context, block| {
    // `(x) = v` is the scalar `x = v`; a multi-element (or blank-bearing) tuple
    // on the left is a destructuring assignment.
    let left = node.left_operand().unwrap_parentheses();
    let right = node.right_operand();
    if let Expression::TupleExpression(tuple) = &left {
        let (bindings, block) = AssignmentTarget::destructure(context, tuple, &right, block);
        let (value, block) = AssignmentTarget::store_all(context, bindings, block);
        return BlockAnd { block, value: value.into() };
    }

    let (target, block) = AssignmentTarget::new(context, &left, block);
    let (value, block) = if matches!(node.operator(), ast::AssignmentExpressionOperator::Equal(_)) {
        // A string literal assigned to a `bytesN` / `byte` value lvalue is a
        // fixed-bytes constant; emit it toward the target's element type so it
        // does not become a runtime `sol.string` the store would reject.
        match &target {
            AssignmentTarget::Pointer(_, element_type)
            | AssignmentTarget::Storage(_, element_type) => {
                let BlockAnd { value, block } = right.emit_as(*element_type, context, block);
                (value, block)
            }
            AssignmentTarget::ReferenceCopy(_) => {
                let BlockAnd { value, block } = right.emit(context, block);
                (value, block)
            }
        }
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
            AssignmentTarget::ReferenceCopy(_) => unreachable!(
                "a compound assignment to a reference-typed lvalue is rejected by the type checker"
            ),
        };
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        // Shares the binary-operation emitter with `a op b` so a compound bitwise assignment
        // on a `bytesN` / `byte` lvalue gets the same fixed-bytes bridge.
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
