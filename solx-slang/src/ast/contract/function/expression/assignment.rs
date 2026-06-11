//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::TypeConversion;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
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

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`).
    pub fn emit_assignment(
        &self,
        assign: &slang_solidity_v2::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // `(x) = v` is the scalar `x = v`; a multi-element (or blank-bearing)
        // tuple on the left is a destructuring assignment.
        let left = Self::unwrap_parenthesised(assign.left_operand());
        let right = assign.right_operand();
        if let Expression::TupleExpression(tuple) = &left {
            return self.emit_tuple_assignment(tuple, &right, block);
        }

        let (target, block) = self.resolve_assignment_target(&left, block)?;
        let (value, block) = if matches!(
            assign.operator(),
            ast::AssignmentExpressionOperator::Equal(_)
        ) {
            // A string literal assigned to a `bytesN` / `byte` value lvalue is a
            // fixed-bytes constant; emit it toward the target's element type so
            // it does not become a runtime `sol.string` the store would reject.
            match &target {
                AssignmentTarget::Pointer(_, element_type)
                | AssignmentTarget::Storage(_, element_type) => {
                    self.emit_value_for_target(&right, *element_type, block)?
                }
                AssignmentTarget::ReferenceCopy(_) => self.emit_value(&right, block)?,
            }
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
                AssignmentTarget::Pointer(pointer, element_type) => {
                    let old = self
                        .state
                        .builder
                        .emit_sol_load(*pointer, *element_type, &block)?;
                    (old, *element_type)
                }
                AssignmentTarget::Storage(slot, element_type) => {
                    let old = self.emit_storage_load(slot, *element_type, &block)?;
                    (old, *element_type)
                }
                AssignmentTarget::ReferenceCopy(_) => unreachable!(
                    "a compound assignment to a reference-typed lvalue is rejected by the type checker"
                ),
            };
            let (rhs, block) = self.emit_value(&right, block)?;
            // Shares the binary-operation emitter with `a op b` so a compound
            // bitwise assignment (`a ^= b`, `a <<= b`) on a `bytesN` / `byte`
            // lvalue gets the same fixed-bytes bridge.
            let result = self.emit_value_binary_operation(operator, old, rhs, target_type, &block);
            (result, block)
        };

        let result = self.store_into_target(&target, value, &block);
        Ok((result, block))
    }

    /// Emits `delete x` — resets the lvalue to its zero value. A reference-typed
    /// storage aggregate (array / `string` / `bytes` / struct) is deep-cleared
    /// via `sol.delete`; a value-typed lvalue (scalar state variable, `a[i]`,
    /// `m[k]`) is overwritten with zero, reusing the assignment store path.
    pub fn emit_delete(
        &self,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (target, block) = self.resolve_assignment_target(operand, block)?;
        match &target {
            AssignmentTarget::ReferenceCopy(reference) => {
                self.state.builder.emit_sol_delete(*reference, &block);
            }
            AssignmentTarget::Pointer(_, element_type)
            | AssignmentTarget::Storage(_, element_type) => {
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
                            let size = self.state.builder.emit_sol_constant(
                                0,
                                self.state.builder.types.ui256,
                                &block,
                            );
                            self.state.builder.emit_sol_malloc_sized_zeroed(
                                *element_type,
                                size,
                                &block,
                            )
                        }
                        _ => self
                            .state
                            .builder
                            .emit_sol_malloc_zeroed(*element_type, &block),
                    }
                } else {
                    // The zero of a value lvalue is its type's own zero, not a raw
                    // `ui256` 0: a function pointer resets to `default_func_constant`,
                    // an address to `address(0)`, a `bytesN`/enum to its typed zero.
                    // Emitting a `ui256` 0 and letting the store coerce it would
                    // `sol.cast` it to e.g. a `func_ref` (an ill-typed integer cast).
                    TypeConversion::emit_scalar_zero(
                        &slang_type,
                        *element_type,
                        &self.state.builder,
                        &block,
                    )
                };
                self.store_into_target(&target, zero, &block);
            }
        }
        Ok(block)
    }

    /// Resolves a state-variable lvalue — bare `x` or namespace-qualified `C.x`
    /// (e.g. a function-pointer state variable) — to its assignment target.
    ///
    /// Reference-typed storage (fixed/dynamic arrays, `string`, `bytes`,
    /// structs) is assigned by copying the RHS reference's contents into the
    /// slot via `sol.copy` (a [`ReferenceCopy`](AssignmentTarget::ReferenceCopy)),
    /// just like a reference-typed inline initializer; a whole mapping is not
    /// assignable. Value-typed storage stores the coerced scalar directly.
    fn resolve_state_variable_target(
        &self,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        AssignmentTarget<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let declared_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            })
            .clone();
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        if declared_type.is_reference_type() && !matches!(declared_type, ast::Type::Mapping(_)) {
            let address_type = Self::address_type(
                &self.state.builder,
                element_type,
                slot.location,
                &declared_type,
            );
            let storage_ref = self
                .state
                .builder
                .emit_sol_addr_of(&slot.name, address_type, &block);
            return Ok((AssignmentTarget::ReferenceCopy(storage_ref), block));
        }
        Ok((AssignmentTarget::Storage(slot, element_type), block))
    }

    /// Resolves a single left-hand-side expression to its assignment target.
    fn resolve_assignment_target(
        &self,
        target_expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        AssignmentTarget<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let resolved = match target_expression {
            Expression::Identifier(identifier) => {
                let target = match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        return self.resolve_state_variable_target(&state_variable, block);
                    }
                    Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                        let (pointer, element_type) =
                            self.environment.variable_with_type(definition.node_id());
                        AssignmentTarget::Pointer(pointer, element_type)
                    }
                    None => unreachable!("slang resolves every identifier reference"),
                    Some(other) => unimplemented!(
                        "assignment to non-variable definition {:?} is not yet supported",
                        other.node_id()
                    ),
                };
                (target, block)
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                if address.r#type() == element_type {
                    // A reference-typed element is addressed by reference (the
                    // address value's type IS the element type), so it is copied
                    // into rather than stored over.
                    (AssignmentTarget::ReferenceCopy(address), block)
                } else {
                    (AssignmentTarget::Pointer(address, element_type), block)
                }
            }
            Expression::MemberAccessExpression(access) => {
                // A namespace-qualified state-variable lvalue (`C.x = v`, notably
                // a function-pointer state variable) is not a struct field;
                // resolve it to its storage target exactly like the bare `x = v`.
                // A genuine struct field resolves to a `StructMember`, so it falls
                // through to the field-address path.
                if let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                {
                    return self.resolve_state_variable_target(&state_variable, block);
                }
                let (address, element_type, block) =
                    self.emit_struct_field_address(access, block)?;
                if address.r#type() == element_type {
                    // A reference-typed field is addressed by reference (the
                    // address value's type IS the element type), so it is copied
                    // into rather than stored over.
                    (AssignmentTarget::ReferenceCopy(address), block)
                } else {
                    (AssignmentTarget::Pointer(address, element_type), block)
                }
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
                // reference to it; resolve that reference as the lvalue so the RHS
                // is stored into the freshly-appended slot (like `arr.push(v)`).
                let Expression::MemberAccessExpression(access) = call.operand() else {
                    unreachable!("guarded by the match arm");
                };
                let (new_slot, element_type, block) =
                    CallEmitter::new(self).emit_push_slot(&access, block)?;
                if new_slot.r#type() == element_type {
                    // A reference-typed element is addressed by reference, so the
                    // RHS reference's contents are copied into it.
                    (AssignmentTarget::ReferenceCopy(new_slot), block)
                } else {
                    (AssignmentTarget::Pointer(new_slot, element_type), block)
                }
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(target_expression)
            ),
        };
        Ok(resolved)
    }

    /// Stores a coerced value into a resolved assignment target.
    fn store_into_target(
        &self,
        target: &AssignmentTarget<'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match target {
            AssignmentTarget::Pointer(pointer, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    *element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, block);
                self.state
                    .builder
                    .emit_sol_store(stored_value, *pointer, block);
                stored_value
            }
            AssignmentTarget::Storage(slot, element_type) => {
                let stored_value = TypeConversion::from_target_type(
                    *element_type,
                    &self.state.builder,
                )
                .emit(value, &self.state.builder, block);
                self.emit_storage_store(slot, stored_value, *element_type, block);
                stored_value
            }
            AssignmentTarget::ReferenceCopy(address) => {
                // The RHS is already a reference of the matching type; copy its
                // contents into the destination reference (no scalar coercion).
                self.state.builder.emit_sol_copy(value, *address, block);
                value
            }
        }
    }

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
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // Materialise the assignment as `(lvalue, value)` pairs, evaluating
        // every value before any store (Solidity's `(a, b) = (b, a)` swap).
        let (assignments, mut block): (Vec<(Expression, Value<'context, 'block>)>, _) = match right
        {
            Expression::TupleExpression(rhs_tuple) => {
                // Pair LHS lvalues with RHS value expressions, recursing only
                // where BOTH sides are tuples — so a blank slot
                // (`(a, ) = (4, (8, 16, 32))`) discards the whole nested tuple
                // rather than spreading it across slots.
                let pairs = Self::pair_tuple_assignment(tuple, rhs_tuple);
                let mut assignments = Vec::new();
                let mut current = block;
                for (lvalue, rhs_expression) in pairs {
                    match lvalue {
                        Some(lvalue) => {
                            let (value, next) = self.emit_value(&rhs_expression, current)?;
                            current = next;
                            assignments.push((lvalue, value));
                        }
                        // A discarded scalar is still evaluated for its side
                        // effects; a discarded nested tuple is dropped wholesale.
                        None if !matches!(rhs_expression, Expression::TupleExpression(_)) => {
                            let (_discarded, next) = self.emit_value(&rhs_expression, current)?;
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
                let lhs_leaves = Self::flatten_tuple_lvalues(tuple);
                let (values, current) =
                    CallEmitter::new(self).emit_function_call_results(call, block)?;
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
                let lhs_leaves = Self::flatten_tuple_lvalues(tuple);
                let (values, current) = self.emit_conditional_tuple_values(conditional, block)?;
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
            let (target, next) = self.resolve_assignment_target(&lvalue, block)?;
            block = next;
            targets.push((target, value));
        }

        let mut result = None;
        for (target, value) in targets.into_iter().rev() {
            result = Some(self.store_into_target(&target, value, &block));
        }
        // A fully blank LHS `(, ) = f()` binds nothing; the assignment still has
        // a value in expression position, so fall back to a zero sentinel.
        let result = result.unwrap_or_else(|| {
            self.state
                .builder
                .emit_sol_constant(0, self.state.builder.types.ui256, &block)
        });
        Ok((result, block))
    }

    /// Pairs a tuple LHS with a tuple RHS into `(lvalue, value-expression)`
    /// pairs, recursing into nested tuples only where BOTH sides nest, so a
    /// blank LHS slot opposite a nested RHS tuple discards it as a unit. A blank
    /// slot yields `None` for its lvalue.
    fn pair_tuple_assignment(
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
                ) => {
                    pairs.extend(Self::pair_tuple_assignment(lhs_nested, rhs_nested));
                }
                _ => pairs.push((lhs_expression, rhs_expression)),
            }
        }
        pairs
    }

    /// Flattens a tuple's left-hand-side leaves, recursing into nested tuples
    /// (`(a, (b, c))` -> `[a, b, c]`). A blank slot is `None` (discarded). Used
    /// for call / conditional right-hand sides, whose values are already flat.
    fn flatten_tuple_lvalues(tuple: &TupleExpression) -> Vec<Option<Expression>> {
        let mut leaves = Vec::new();
        for item in tuple.items().iter() {
            match item.expression() {
                Some(Expression::TupleExpression(nested)) => {
                    leaves.extend(Self::flatten_tuple_lvalues(&nested));
                }
                other => leaves.push(other),
            }
        }
        leaves
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

    /// Peels parenthesised single-element tuples so `((x))` resolves to `x`.
    fn unwrap_parenthesised(expression: Expression) -> Expression {
        let mut expression = expression;
        while let Expression::TupleExpression(tuple) = &expression {
            let items = tuple.items();
            if items.len() != 1 {
                break;
            }
            let Some(inner) = items
                .iter()
                .next()
                .expect("a one-element tuple has its element")
                .expression()
            else {
                break;
            };
            expression = inner;
        }
        expression
    }
}
