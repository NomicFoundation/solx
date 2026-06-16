//!
//! Assignment expression emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;
use solx_mlir::ods::sol::AddrOfOperation;
use solx_mlir::ods::sol::CopyOperation;
use solx_mlir::ods::sol::DeleteOperation;
use solx_mlir::ods::sol::MallocOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::EmitAddress;
use crate::ast::LocationPolicy;
use crate::ast::Materialize;
use crate::ast::Place;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::contract::storage_layout::StorageSlot;

/// Assignment target resolved from the Slang binder.
pub enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot and declared element type.
    Storage(StorageSlot, Type<'context>),
    /// Reference-typed location (array/struct/`string`/`bytes` addressed by
    /// reference, in storage or calldata): the destination reference into
    /// which the RHS reference's contents are copied via `sol.copy`.
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
                    let pointer = crate::ast::Pointer::new(
                        context.environment.variable(definition.node_id()),
                    );
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
                } = index_access.emit_address(context, block);
                (Self::from_address(address, element_type), block)
            }
            Expression::MemberAccessExpression(access) => {
                // A namespace-qualified state-variable lvalue (`C.x = v`) is not a
                // struct field; resolve it like the bare `x = v`. A genuine struct
                // field resolves to a `StructMember`, falling through below.
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
                } = access.emit_address(context, block);
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
                let (new_slot, element_type, block) = context.emit_push_slot(&access, block);
                (Self::from_address(new_slot, element_type), block)
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(target_expression)
            ),
        }
    }

    /// Resolves a state-variable lvalue — bare `x` or namespace-qualified `C.x`
    /// — to its assignment target.
    ///
    /// Reference-typed storage (arrays, `string`, `bytes`, structs) is assigned
    /// by copying the RHS reference's contents into the slot via `sol.copy` (a
    /// [`Self::ReferenceCopy`]); a whole mapping is not assignable. Value-typed
    /// storage stores the coerced scalar directly.
    fn from_state_variable<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Self, BlockRef<'context, 'block>) {
        let declared_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let slot = context
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            })
            .clone();
        let element_type = crate::ast::Type::resolve(
            &declared_type,
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        if declared_type.is_reference_type() && !matches!(declared_type, ast::Type::Mapping(_)) {
            let address_type = crate::ast::Type::new(element_type)
                .address_type(slot.location, context.state.builder.context)
                .into_mlir();
            let storage_ref = sol_op!(
                &context.state.builder,
                &block,
                AddrOfOperation
                    .var(FlatSymbolRefAttribute::new(
                        context.state.builder.context,
                        &slot.name,
                    ))
                    .addr(address_type)
            );
            return (Self::ReferenceCopy(storage_ref), block);
        }
        (Self::Storage(slot, element_type), block)
    }

    /// Classifies a computed lvalue `address` (with its `element_type`) into its
    /// assignment target. A reference-typed element is addressed BY reference —
    /// the address value's type IS the element type — so it is copied into
    /// ([`Self::ReferenceCopy`]); any other element is a [`Self::Pointer`] stored
    /// through. Shared by the index, struct-field, and `push()` lvalue paths.
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
                let stored_value = crate::ast::Value::from(value).cast(
                    crate::ast::Type::new(*element_type),
                    &context.state.builder,
                    block,
                );
                crate::ast::Pointer::new(*pointer).store(
                    stored_value,
                    &context.state.builder,
                    block,
                );
                stored_value.into_mlir()
            }
            Self::Storage(slot, element_type) => {
                let stored_value = crate::ast::Value::from(value)
                    .cast(
                        crate::ast::Type::new(*element_type),
                        &context.state.builder,
                        block,
                    )
                    .into_mlir();
                slot.store(&context.state.builder, stored_value, *element_type, block);
                stored_value
            }
            Self::ReferenceCopy(address) => {
                // The RHS is already a reference of the matching type; copy its
                // contents into the destination reference (no scalar coercion).
                sol_op_void!(
                    &context.state.builder,
                    block,
                    CopyOperation.src(value).dst(*address)
                );
                value
            }
        }
    }

    /// Emits `delete x` — resets the lvalue `operand` denotes to its zero value.
    /// A reference-typed storage aggregate is deep-cleared via `sol.delete`; a
    /// memory aggregate resets to a freshly allocated zero-filled buffer; a value
    /// lvalue is overwritten with its type's own zero (reusing the store path).
    pub fn delete<'state>(
        context: &ExpressionContext<'state, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let (target, block) = Self::new(context, operand, block);
        match &target {
            Self::ReferenceCopy(reference) => {
                sol_op_void!(
                    &context.state.builder,
                    &block,
                    DeleteOperation.reference(*reference)
                );
            }
            Self::Pointer(_, element_type) | Self::Storage(_, element_type) => {
                let slang_type = operand
                    .get_type()
                    .expect("slang types every delete operand");
                let zero = if slang_type.is_reference_type() {
                    // A memory aggregate (array / struct / `string` / `bytes`)
                    // resets to a freshly allocated zero-filled buffer
                    // (`sol.malloc zero_init`), exactly as it is default-
                    // initialised — not a scalar zero. (A storage aggregate is
                    // deep-cleared via the `ReferenceCopy` arm above.)
                    match &slang_type {
                        ast::Type::String(_) | ast::Type::Bytes(_) => {
                            let size = crate::ast::Value::constant(
                                0,
                                crate::ast::Type::unsigned(
                                    context.state.builder.context,
                                    solx_utils::BIT_LENGTH_FIELD,
                                ),
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir();
                            sol_op!(
                                &context.state.builder,
                                &block,
                                MallocOperation
                                    .addr(*element_type)
                                    .size(size)
                                    .zero_init(Attribute::unit(context.state.builder.context))
                            )
                        }
                        _ => sol_op!(
                            &context.state.builder,
                            &block,
                            MallocOperation
                                .addr(*element_type)
                                .zero_init(Attribute::unit(context.state.builder.context))
                        ),
                    }
                } else {
                    // The zero of a value lvalue is its type's own zero, not a raw
                    // `ui256` 0: a function pointer resets to `default_func_constant`,
                    // an address to `address(0)`, a `bytesN`/enum to its typed zero.
                    // Emitting a `ui256` 0 and letting the store coerce it would
                    // `sol.cast` it to e.g. a `func_ref` (an ill-typed integer cast).
                    crate::ast::Value::zero(
                        crate::ast::Type::new(*element_type),
                        &context.state.builder,
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

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a destructuring assignment `(a, b, …) = rhs` to existing lvalues.
    ///
    /// Solidity evaluates every right-hand component before writing any
    /// destination (so `(a, b) = (b, a)` swaps), so all RHS values are
    /// materialised first; destinations are then resolved left-to-right against
    /// pre-assignment state and written right-to-left, so the leftmost write to
    /// an aliased destination wins: `(y, y, y) = (1, 2, 3)` leaves `y == 1`, and
    /// a storage-aggregate swap `(x, y) = (y, x)` copies references in place
    /// (both end equal). A blank slot `(, b)` discards its value.
    fn emit_tuple_assignment(
        &self,
        tuple: &TupleExpression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        // Pairs LHS lvalue slots with RHS value expressions, recursing only
        // where BOTH sides nest — a blank LHS slot opposite a nested RHS tuple
        // discards it as a unit. A blank slot yields `None` for its lvalue.
        fn pair_assignment(
            lhs: &TupleExpression,
            rhs: &TupleExpression,
        ) -> Vec<(Option<Expression>, Expression)> {
            let lhs_items = lhs.items();
            let rhs_items = rhs.items();
            assert!(
                lhs_items.len() == rhs_items.len(),
                "tuple assignment arity mismatch: {} LHS slots vs {} RHS values",
                lhs_items.len(),
                rhs_items.len(),
            );
            let mut pairs = Vec::new();
            for (lhs_item, rhs_item) in lhs_items.iter().zip(rhs_items.iter()) {
                let lhs_expression = lhs_item.expression();
                let rhs_expression = rhs_item
                    .expression()
                    .expect("a tuple assignment RHS element has an inner expression");
                match (&lhs_expression, &rhs_expression) {
                    (
                        Some(Expression::TupleExpression(lhs_nested)),
                        Expression::TupleExpression(rhs_nested),
                    ) => pairs.extend(pair_assignment(lhs_nested, rhs_nested)),
                    _ => pairs.push((lhs_expression, rhs_expression)),
                }
            }
            pairs
        }

        // Flattens LHS lvalue leaves, recursing into nested tuples
        // (`(a, (b, c))` -> `[a, b, c]`); a blank slot is `None` (discarded). For
        // a call / conditional RHS, whose values are already flat.
        fn flatten_lvalues(tuple: &TupleExpression) -> Vec<Option<Expression>> {
            let mut leaves = Vec::new();
            for item in tuple.items().iter() {
                match item.expression() {
                    Some(Expression::TupleExpression(nested)) => {
                        leaves.extend(flatten_lvalues(&nested))
                    }
                    other => leaves.push(other),
                }
            }
            leaves
        }

        // Materialise the assignment as `(lvalue, value)` pairs, evaluating
        // every value before any store (Solidity's `(a, b) = (b, a)` swap).
        let (assignments, mut block): (Vec<(Expression, Value<'context, 'block>)>, _) = match right
        {
            Expression::TupleExpression(rhs_tuple) => {
                // Pair LHS lvalues with RHS value expressions, recursing only
                // where BOTH sides are tuples — so a blank slot
                // (`(a, ) = (4, (8, 16, 32))`) discards the whole nested tuple
                // rather than spreading it across slots.
                let pairs = pair_assignment(tuple, rhs_tuple);
                let mut assignments = Vec::new();
                let mut current = block;
                for (lvalue, rhs_expression) in pairs {
                    match lvalue {
                        Some(lvalue) => {
                            let BlockAnd { value, block: next } =
                                rhs_expression.emit(self, current);
                            current = next;
                            assignments.push((lvalue, value.into_mlir()));
                        }
                        // A discarded scalar is still evaluated for its side
                        // effects; a discarded nested tuple is dropped wholesale.
                        None if !matches!(rhs_expression, Expression::TupleExpression(_)) => {
                            let BlockAnd {
                                value: _discarded,
                                block: next,
                            } = rhs_expression.emit(self, current);
                            current = next;
                        }
                        None => {}
                    }
                }
                (assignments, current)
            }
            // A call / conditional yields a flat value list, so the LHS pairs by
            // flattened leaf (no syntactic nesting can match these).
            Expression::FunctionCallExpression(call) => {
                let lhs_leaves = flatten_lvalues(tuple);
                let (values, current) = self.emit_function_call_results(call, block);
                assert!(
                    values.len() == lhs_leaves.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} call results",
                    lhs_leaves.len(),
                    values.len(),
                );
                (Self::zip_assignments(lhs_leaves, values), current)
            }
            Expression::ConditionalExpression(conditional) => {
                // `(a, b) = cond ? (x, y) : (z, w)` — the conditional yields one
                // value per tuple element via the shared tuple-conditional path.
                let lhs_leaves = flatten_lvalues(tuple);
                let (values, current) = self.emit_conditional_tuple_values(conditional, block);
                assert!(
                    values.len() == lhs_leaves.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} conditional values",
                    lhs_leaves.len(),
                    values.len(),
                );
                (Self::zip_assignments(lhs_leaves, values), current)
            }
            _ => unimplemented!(
                "tuple assignment with this right-hand side shape is not yet supported"
            ),
        };

        // Resolve every LHS lvalue address first, left-to-right against the
        // pre-assignment state, then store RIGHT-TO-LEFT: invisible for value
        // types, but reproducing Solidity's storage-aggregate quirk that a
        // `(x, y) = (y, x)` swap does not work and that the leftmost write to an
        // aliased destination wins (`(y, y, y) = (1, 2, 3)` leaves `y == 1`).
        let mut targets = Vec::with_capacity(assignments.len());
        for (lvalue, value) in assignments {
            let (target, next) = AssignmentTarget::new(self, &lvalue, block);
            block = next;
            targets.push((target, value));
        }

        let mut result = None;
        for (target, value) in targets.into_iter().rev() {
            result = Some(target.store(self, value, &block));
        }
        // A fully blank LHS `(, ) = f()` binds nothing; the assignment still has
        // a value in expression position, so fall back to a zero sentinel.
        let result = result.unwrap_or_else(|| {
            crate::ast::Value::constant(
                0,
                crate::ast::Type::unsigned(
                    self.state.builder.context,
                    solx_utils::BIT_LENGTH_FIELD,
                ),
                &self.state.builder,
                &block,
            )
            .into_mlir()
        });
        (result, block)
    }

    /// Zips flattened LHS leaves with their values, dropping blank slots.
    fn zip_assignments(
        lhs_leaves: Vec<Option<Expression>>,
        values: Vec<Value<'context, 'block>>,
    ) -> Vec<(Expression, Value<'context, 'block>)> {
        lhs_leaves
            .into_iter()
            .zip(values)
            .filter_map(|(lvalue, value)| lvalue.map(|lvalue| (lvalue, value)))
            .collect()
    }
}

// An assignment expression (`=`, `+=`, `-=`, `*=`, …).
expression_emit!(AssignmentExpression; |node, context, block| {
    // `(x) = v` is the scalar `x = v`; a multi-element (or blank-bearing) tuple
    // on the left is a destructuring assignment.
    let left = node.left_operand().unwrap_parentheses();
    let right = node.right_operand();
    if let Expression::TupleExpression(tuple) = &left {
        let (value, block) = context.emit_tuple_assignment(tuple, &right, block);
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
                let BlockAnd { value, block } =
                    if let Expression::StringExpression(string_literal) = &right {
                        string_literal.materialize(*element_type, context, block)
                    } else {
                        right.emit(context, block)
                    };
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
                let old = crate::ast::Pointer::new(*pointer).load(
                    crate::ast::Type::new(*element_type),
                    &context.state.builder,
                    &block,
                );
                (old, *element_type)
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let old = slot.load(&context.state.builder, *element_type, &block);
                (crate::ast::Value::from(old), *element_type)
            }
            AssignmentTarget::ReferenceCopy(_) => unreachable!(
                "a compound assignment to a reference-typed lvalue is rejected by the type checker"
            ),
        };
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        // Shares the binary-operation emitter with `a op b` so a compound
        // bitwise assignment (`a ^= b`, `a <<= b`) on a `bytesN` / `byte`
        // lvalue gets the same fixed-bytes bridge.
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
