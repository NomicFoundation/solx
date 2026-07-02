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
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::DeleteOperation;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_place::EmitPlace;
use crate::ast::emit::emit_values::EmitValues;

/// Assignment target resolved from the Slang binder.
pub enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
    /// Reference-typed location: the destination into which the RHS reference's contents are copied via `sol.copy`.
    ReferenceCopy(Value<'context, 'block>),
}

impl<'context: 'block, 'block> AssignmentTarget<'context, 'block> {
    /// Resolves a single left-hand-side expression to its assignment target.
    fn new<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        target_expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        match target_expression {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        return Self::from_state_variable(context, &state_variable, block);
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
                (Self::from_address(place.address, place.element_type), block)
            }
            Expression::MemberAccessExpression(access) => {
                if let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                {
                    return Self::from_state_variable(context, &state_variable, block);
                }
                let BlockAnd {
                    value: place,
                    block,
                } = access.emit_place(context, block);
                (Self::from_address(place.address, place.element_type), block)
            }
            Expression::FunctionCallExpression(call)
                if matches!(
                    call.operand().unwrap_parentheses(),
                    Expression::MemberAccessExpression(access)
                        if matches!(access.member().resolve_to_built_in(), Some(BuiltIn::ArrayPush))
                ) =>
            {
                let Expression::MemberAccessExpression(access) =
                    call.operand().unwrap_parentheses()
                else {
                    unreachable!("guarded by the match arm");
                };
                let element_type = context
                    .resolve_slang_type(call.get_type())
                    .expect("slang types every array push");
                let (slot, block) =
                    CallContext::new(context).emit_array_push(&access, None, block);
                let slot = slot.expect("a no-argument array push yields its new slot");
                (Self::from_address(slot, element_type), block)
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(target_expression)
            ),
        }
    }

    /// Resolves a state-variable lvalue (bare `x` or namespace-qualified `C.x`) to its target.
    /// Reference-typed storage is copied via `sol.copy` ([`Self::ReferenceCopy`]); value-typed
    /// storage stores the scalar directly ([`Self::Storage`]).
    fn from_state_variable<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        let declared_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        let slot = context
            .storage_layout
            .get(&state_variable.node_id())
            .expect("state variable is registered in the storage layout")
            .clone();
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, context.state);
        if declared_type.is_reference_type() && !matches!(declared_type, ast::Type::Mapping(_)) {
            let address_type = ExpressionContext::address_type(
                context.state,
                element_type,
                slot.location,
                &declared_type,
            );
            let storage_ref =
                Pointer::addr_of(&slot.name, AstType::new(address_type), context.state, &block)
                    .into_mlir();
            return (AssignmentTarget::ReferenceCopy(storage_ref), block);
        }
        (AssignmentTarget::Storage(slot, element_type), block)
    }

    /// Classifies a computed lvalue `address` into its target: a reference element, whose address
    /// type is its element type, becomes a [`Self::ReferenceCopy`], any other a [`Self::Pointer`].
    fn from_address(address: Value<'context, 'block>, element_type: Type<'context>) -> Self {
        if address.r#type() == element_type {
            AssignmentTarget::ReferenceCopy(address)
        } else {
            AssignmentTarget::Pointer(address, element_type)
        }
    }

    /// Coerces `value` to this target's element type and stores it, returning the stored value.
    ///
    /// A reference-typed target copies the RHS reference's contents in via `sol.copy` rather than
    /// coercing and scalar-storing.
    fn store<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match self {
            AssignmentTarget::Pointer(pointer, element_type) => {
                let stored_value = TypeConversion::from_target_type(*element_type, context.state)
                    .emit(value, context.state, block);
                Pointer::new(*pointer).store(AstValue::new(stored_value), context.state, block);
                stored_value
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value = TypeConversion::from_target_type(*element_type, context.state)
                    .emit(value, context.state, block);
                context.emit_storage_store(slot, stored_value, *element_type, block);
                stored_value
            }
            AssignmentTarget::ReferenceCopy(address) => {
                Pointer::new(*address).copy_from(AstValue::new(value), context.state, block);
                value
            }
        }
    }

    /// Collects the `(lvalue, value)` bindings of a destructuring assignment `(a, b, …) = rhs`,
    /// evaluating every value before any store, so `(a, b) = (b, a)` swaps. A blank slot discards
    /// its right-hand-side element.
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
                    let rhs = rhs.expression().expect("slang validates tuple element");
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
                            bindings.push((lvalue, value));
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
                } = rhs.emit_values(context, block);
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

    /// Resolves each lvalue left-to-right against the pre-assignment state, then stores right-to-left
    /// so the leftmost write to an aliased destination wins. Returns the last stored value, or a zero
    /// when every slot is blank.
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
            .unwrap_or_else(|| {
                let field_type =
                    AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD);
                AstValue::constant(0, field_type, context.state, &block).into_mlir()
            });
        (result, block)
    }

    /// Emits `delete x` — resets the lvalue to its zero. A reference-typed storage aggregate is
    /// deep-cleared via `sol.delete`; a reference-typed memory aggregate resets to a fresh
    /// zero-filled buffer; a value lvalue resets to its typed zero.
    pub fn delete<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let (target, block) = Self::new(context, operand, block);
        match &target {
            AssignmentTarget::ReferenceCopy(reference) => {
                mlir_op_void!(context.state, &block, DeleteOperation.reference(*reference));
            }
            AssignmentTarget::Pointer(_, element_type)
            | AssignmentTarget::Storage(_, element_type) => {
                let slang_type = operand.get_type().expect("slang types every lvalue");
                let zero = if slang_type.is_reference_type() {
                    let zero_init =
                        !matches!(slang_type, ast::Type::String(_) | ast::Type::Bytes(_));
                    AstValue::malloc(
                        AstType::new(*element_type),
                        None,
                        zero_init,
                        context.state,
                        &block,
                    )
                    .into_mlir()
                } else if AstType::new(*element_type).is_function_ref() {
                    AstValue::function_pointer_zero(
                        AstType::new(*element_type),
                        context.state,
                        &block,
                    )
                    .into_mlir()
                } else {
                    AstValue::constant(
                        0,
                        AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                        context.state,
                        &block,
                    )
                    .into_mlir()
                };
                target.store(context, zero, &block);
            }
        }
        block
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for AssignmentExpression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let left = self.left_operand().unwrap_parentheses();
        let right = self.right_operand();
        if let Expression::TupleExpression(tuple) = &left {
            let (bindings, block) = AssignmentTarget::destructure(context, tuple, &right, block);
            let (value, block) = AssignmentTarget::store_all(context, bindings, block);
            return BlockAnd { block, value };
        }

        let (target, block) = AssignmentTarget::new(context, &left, block);
        let (value, block) = if matches!(
            self.operator(),
            ast::AssignmentExpressionOperator::Equal(_)
        ) {
            match (&target, &right) {
                (
                    AssignmentTarget::Pointer(_, element_type)
                    | AssignmentTarget::Storage(_, element_type),
                    Expression::StringExpression(string_literal),
                ) if context.is_byte(*element_type) => {
                    let BlockAnd { value, block } =
                        string_literal.emit_as(*element_type, context, block);
                    (value, block)
                }
                _ => {
                    let BlockAnd { value, block } = right.emit(context, block);
                    (value, block)
                }
            }
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
                AssignmentTarget::ReferenceCopy(_) => unreachable!(
                    "a compound assignment to a reference-typed lvalue is rejected by the type checker"
                ),
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

        let value = target.store(context, value, &block);
        BlockAnd { block, value }
    }
}
