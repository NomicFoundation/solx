//!
//! Assignment expression lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use ruint::aliases::U256;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::ExpressionExt;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// Assignment target resolved from the Slang binder.
enum AssignmentTarget<'context, 'block> {
    /// Address-typed pointer with its declared element type.
    ///
    /// Covers local variables, function parameters, and the result of an
    /// `a[i]` / `m[k]` index-access expression on the left-hand side.
    Pointer(Value<'context, 'block>, Type<'context>),
    /// State variable — storage slot, byte offset within the slot, declared
    /// element type, and data location (`Storage` or `Transient`).
    Storage(U256, u32, Type<'context>, solx_utils::DataLocation),
}

/// Resolved left-hand side of an assignment: either a value-typed location to
/// store into, or a reference-typed location to copy the RHS reference into.
enum LvalueTarget<'context, 'block> {
    /// Value-typed location: coerce the RHS and `sol.store` / storage-store it.
    Store(AssignmentTarget<'context, 'block>),
    /// Reference-typed location (array/struct/string/bytes addressed by
    /// reference): copy the RHS reference's contents in via `sol.copy`.
    ReferenceCopy(Value<'context, 'block>),
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an assignment expression (`=`, `+=`, `-=`, `*=`, tuple `=`).
    pub fn emit_assignment(
        &self,
        assign: &slang_solidity_v2::ast::AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let left = assign.left_operand();

        // A parenthesised single-element tuple LHS (`(x) = v`, or nested
        // `(((x))) = v`) is a decayed tuple — semantically `x = v`. Peel all such
        // tuples so it resolves as a scalar lvalue rather than taking the
        // tuple-assignment path (which expects a tuple/call right-hand side).
        let left = left.unwrap_parens();

        // `(a, b, ...) = rhs` — tuple / destructuring assignment. Only the plain
        // `=` operator is valid on a tuple left-hand side.
        if let Expression::TupleExpression(tuple) = &left
            && matches!(assign.operator(), ast::AssignmentExpressionOperator::Equal(_))
        {
            return self.emit_tuple_assignment(tuple, &assign.right_operand(), block);
        }

        let (target, block) = self.resolve_lvalue(&left, block)?;

        // Plain assignment: evaluate the RHS and store / copy it into the target.
        if matches!(assign.operator(), ast::AssignmentExpressionOperator::Equal(_)) {
            // Emit the RHS already coerced toward the lvalue's element type, so a
            // string literal assigned to a `bytesN` / byte element materializes
            // as a constant of that type (via `emit_value_for_target`) instead of
            // a dynamic string that `sol.cast` / `sol.bytes_cast` then reject.
            // For reference-copy targets there is no scalar element type to
            // coerce toward, so fall back to a plain `emit_value`.
            let store_element_type = match &target {
                LvalueTarget::Store(AssignmentTarget::Pointer(_, element_type)) => {
                    Some(*element_type)
                }
                LvalueTarget::Store(AssignmentTarget::Storage(_, _, element_type, _)) => {
                    Some(*element_type)
                }
                LvalueTarget::ReferenceCopy(_) => None,
            };
            let (value, block) = match store_element_type {
                Some(element_type) => {
                    self.emit_value_for_target(&assign.right_operand(), element_type, block)?
                }
                None => self.emit_value(&assign.right_operand(), block)?,
            };
            let result = self.store_to_lvalue(target, value, &block);
            return Ok((result, block));
        }

        // Compound assignment (`+=`, `-=`, ...) reads the current value, applies
        // the operator, and writes back — so it needs a value-typed, loadable
        // target. Reference types are never the LHS of a compound operator.
        let LvalueTarget::Store(store_target) = target else {
            unimplemented!("compound assignment to a reference-typed lvalue is not supported");
        };
        let operator = match assign.operator() {
            ast::AssignmentExpressionOperator::AmpersandEqual(_) => Operator::BitwiseAnd,
            ast::AssignmentExpressionOperator::AsteriskEqual(_) => Operator::Multiply,
            ast::AssignmentExpressionOperator::BarEqual(_) => Operator::BitwiseOr,
            ast::AssignmentExpressionOperator::CaretEqual(_) => Operator::BitwiseXor,
            ast::AssignmentExpressionOperator::Equal(_) => {
                unreachable!("`=` is handled above")
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
        let (old, target_type) = match &store_target {
            AssignmentTarget::Pointer(pointer, element_type) => {
                let old = self
                    .state
                    .builder
                    .emit_sol_load(*pointer, *element_type, &block)?;
                (old, *element_type)
            }
            AssignmentTarget::Storage(slot, byte_offset, element_type, location) => {
                let old =
                    self.emit_storage_load(*slot, *byte_offset, *element_type, *location, &block)?;
                (old, *element_type)
            }
        };
        let (rhs, block) = self.emit_value(&assign.right_operand(), block)?;
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
        let result = self.store_to_lvalue(LvalueTarget::Store(store_target), result, &block);
        Ok((result, block))
    }

    /// Resolves a state-variable lvalue — bare `x` or contract-qualified `C.x`
    /// (e.g. a function-pointer state variable) — to its storage target: a
    /// reference copy for reference-typed storage, else a direct storage store.
    /// `name` is used only for diagnostics.
    fn resolve_state_variable_lvalue(
        &self,
        state_variable: &slang_solidity_v2::ast::StateVariableDefinition,
        name: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(LvalueTarget<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("unresolved type for state variable: {name}"))?;
        let &(slot, byte_offset, location) = self
            .storage_layout
            .get(&state_variable.node_id())
            .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        // Reference-typed storage (fixed/dynamic arrays, `string`, `bytes`,
        // structs) is assigned by copying the RHS reference into the slot.
        // Mappings are not assignable.
        if declared_type.is_reference_type()
            && !matches!(declared_type, slang_solidity_v2::ast::Type::Mapping(_))
        {
            let address_type =
                Self::address_type(&self.state.builder, element_type, location, &declared_type);
            let storage_ref = self.state.builder.emit_sol_addr_of(
                &crate::ast::contract::ContractEmitter::storage_symbol(slot, byte_offset, location),
                address_type,
                &block,
            );
            return Ok((LvalueTarget::ReferenceCopy(storage_ref), block));
        }
        Ok((
            LvalueTarget::Store(AssignmentTarget::Storage(
                slot,
                byte_offset,
                element_type,
                location,
            )),
            block,
        ))
    }

    /// Resolves an assignment left-hand side to a [`LvalueTarget`].
    fn resolve_lvalue(
        &self,
        lvalue: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(LvalueTarget<'context, 'block>, BlockRef<'context, 'block>)> {
        match lvalue {
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        self.resolve_state_variable_lvalue(&state_variable, &name, block)
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        Ok((
                            LvalueTarget::Store(AssignmentTarget::Pointer(pointer, element_type)),
                            block,
                        ))
                    }
                    None => unreachable!("slang resolves every identifier reference"),
                    Some(_) => unimplemented!(
                        "assignment to non-variable definition '{name}' is not yet supported"
                    ),
                }
            }
            Expression::IndexAccessExpression(index_access) => {
                let (address, element_type, block) =
                    self.emit_index_access_address(index_access, block)?;
                if address.r#type() == element_type {
                    // Reference-typed element addressed by reference.
                    Ok((LvalueTarget::ReferenceCopy(address), block))
                } else {
                    Ok((
                        LvalueTarget::Store(AssignmentTarget::Pointer(address, element_type)),
                        block,
                    ))
                }
            }
            Expression::MemberAccessExpression(access) => {
                // A contract-qualified state-variable lvalue (`C.x = v`,
                // notably a function-pointer state variable) is not a struct
                // field; resolve it to its storage slot like the bare `x = v`.
                if let Some((address, element_type, block)) =
                    self.emit_struct_field_address(access, block)?
                {
                    if address.r#type() == element_type {
                        // Reference-typed struct field addressed by reference.
                        return Ok((LvalueTarget::ReferenceCopy(address), block));
                    }
                    return Ok((
                        LvalueTarget::Store(AssignmentTarget::Pointer(address, element_type)),
                        block,
                    ));
                }
                if let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                {
                    return self.resolve_state_variable_lvalue(
                        &state_variable,
                        &access.member().name(),
                        block,
                    );
                }
                unimplemented!(
                    "unsupported member-access lvalue: {}",
                    access.member().name()
                )
            }
            Expression::FunctionCallExpression(call)
                if matches!(
                    &call.operand(),
                    Expression::MemberAccessExpression(access)
                        if matches!(
                            access.member().resolve_to_built_in(),
                            Some(slang_solidity_v2::ast::BuiltIn::ArrayPush)
                        )
                ) =>
            {
                // `arr.push() = v` — `push` appends a default element and returns
                // a reference to it; resolve that reference as the lvalue so the
                // right-hand side is stored into the freshly-appended slot
                // (equivalent to `arr.push(v)`).
                let Expression::MemberAccessExpression(access) = call.operand() else {
                    unreachable!("guarded by the match arm")
                };
                let (new_slot, element_type, block) =
                    CallEmitter::new(self).emit_push_slot(&access, block)?;
                if new_slot.r#type() == element_type {
                    Ok((LvalueTarget::ReferenceCopy(new_slot), block))
                } else {
                    Ok((
                        LvalueTarget::Store(AssignmentTarget::Pointer(new_slot, element_type)),
                        block,
                    ))
                }
            }
            _ => unimplemented!(
                "assignment target {:?} is not yet supported",
                std::mem::discriminant(lvalue)
            ),
        }
    }

    /// Writes `value` into a resolved [`LvalueTarget`], returning the value as
    /// stored (coerced to the target's element type for value targets).
    fn store_to_lvalue(
        &self,
        target: LvalueTarget<'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match target {
            LvalueTarget::ReferenceCopy(address) => {
                self.state.builder.emit_sol_copy(value, address, block);
                value
            }
            LvalueTarget::Store(AssignmentTarget::Pointer(pointer, element_type)) => {
                let stored_value = TypeConversion::from_target_type(element_type, &self.state.builder)
                    .emit(value, &self.state.builder, block);
                self.state
                    .builder
                    .emit_sol_store(stored_value, pointer, block);
                stored_value
            }
            LvalueTarget::Store(AssignmentTarget::Storage(
                slot,
                byte_offset,
                element_type,
                location,
            )) => {
                let stored_value = TypeConversion::from_target_type(element_type, &self.state.builder)
                    .emit(value, &self.state.builder, block);
                self.emit_storage_store(slot, byte_offset, stored_value, location, block);
                stored_value
            }
        }
    }

    /// Emits a tuple / destructuring assignment `(a, b, ...) = rhs`.
    ///
    /// Solidity evaluates the entire right-hand side before performing any
    /// assignment (so e.g. `(a, b) = (b, a)` swaps value types), so all RHS
    /// values are materialised first. The LHS lvalue addresses are then resolved
    /// left-to-right (against the pre-assignment state), and the components are
    /// stored RIGHT-TO-LEFT — invisible for value types, but reproducing
    /// Solidity's storage-aggregate quirks (a storage `(x, y) = (y, x)` swap does
    /// not work). Blank components (`(, b) = ...`) discard their value.
    fn emit_tuple_assignment(
        &self,
        tuple: &slang_solidity_v2::ast::TupleExpression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // Materialise the assignment as `(lvalue, value)` pairs, evaluating
        // every value before any store (Solidity's `(a, b) = (b, a)` swap
        // semantics).
        let (assignments, mut block): (Vec<(Expression, Value<'context, 'block>)>, _) = match right
        {
            Expression::TupleExpression(rhs_tuple) => {
                // Pair LHS lvalues with RHS value expressions, recursing only
                // where BOTH sides are tuples — so a blank slot
                // (`(a, ) = (4, (8, 16, 32))`) discards the whole nested tuple
                // rather than spreading it across slots.
                let pairs = Self::pair_tuple_assignment(tuple, rhs_tuple)?;
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
            // A call / conditional yields a flat value list, so the LHS is flat
            // (no syntactic nesting can match these); pair by flattened leaf.
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
                let lhs_leaves = Self::flatten_tuple_lvalues(tuple);
                let (values, current) = self
                    .emit_conditional_tuple_values(conditional, block)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "tuple assignment with this right-hand side shape is not yet supported"
                        )
                    })?;
                assert!(
                    values.len() == lhs_leaves.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} conditional values",
                    lhs_leaves.len(),
                    values.len(),
                );
                (Self::zip_assignments(lhs_leaves, values), current)
            }
            _ => unimplemented!("tuple assignment with this right-hand side shape is not yet supported"),
        };

        // Resolve every LHS lvalue address first, left-to-right, so an index or
        // length is read against the PRE-assignment state: `(s[1], s) = (...)`
        // computes `&s[1]` while `s` still has its original length, rather than
        // after `s` is reassigned and shrunk
        // (array/copying/cleanup_during_multi_element_per_slot_copy).
        let mut targets = Vec::with_capacity(assignments.len());
        for (lvalue, value) in assignments {
            let (target, next) = self.resolve_lvalue(&lvalue, block)?;
            block = next;
            targets.push((target, value));
        }

        // Then store the components RIGHT-TO-LEFT. For value-typed slots the RHS
        // values are already materialised, so the order is irrelevant; for
        // storage AGGREGATES the RHS "value" is a live storage reference (not a
        // copy), so a right-to-left store reproduces Solidity's documented quirk
        // that a storage swap does not work: `(x, y) = (y, x)` runs `y = x` then
        // `x = y`, leaving both equal to the original `x`
        // (various/swap_in_storage_overwrite).
        let mut last_stored = None;
        for (target, value) in targets.into_iter().rev() {
            last_stored = Some(self.store_to_lvalue(target, value, &block));
        }

        // The value of a tuple-assignment expression is rarely consumed; yield
        // the last stored value, or a `0` sentinel if every slot was blank.
        let result = last_stored.unwrap_or_else(|| {
            self.state
                .builder
                .emit_sol_constant(0, self.state.builder.types.ui256, &block)
        });
        Ok((result, block))
    }

    /// Pairs a tuple LHS with a tuple RHS into `(lvalue, value-expression)`
    /// pairs, recursing into nested tuples only where both sides nest, so a
    /// blank LHS slot opposite a nested RHS tuple discards it as a unit. A blank
    /// slot yields `None` for its lvalue.
    fn pair_tuple_assignment(
        lhs: &slang_solidity_v2::ast::TupleExpression,
        rhs: &slang_solidity_v2::ast::TupleExpression,
    ) -> anyhow::Result<Vec<(Option<Expression>, Expression)>> {
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
                .ok_or_else(|| anyhow::anyhow!("empty tuple element on RHS of assignment"))?;
            match (&lhs_expression, &rhs_expression) {
                (
                    Some(Expression::TupleExpression(lhs_nested)),
                    Expression::TupleExpression(rhs_nested),
                ) => {
                    pairs.extend(Self::pair_tuple_assignment(lhs_nested, rhs_nested)?);
                }
                _ => pairs.push((lhs_expression, rhs_expression)),
            }
        }
        Ok(pairs)
    }

    /// Flattens a tuple's left-hand-side leaves, recursing into nested tuples
    /// (`(a, (b, c))` -> `[a, b, c]`). A blank slot is `None` (discarded). Used
    /// for call / conditional right-hand sides, whose values are already flat.
    fn flatten_tuple_lvalues(
        tuple: &slang_solidity_v2::ast::TupleExpression,
    ) -> Vec<Option<Expression>> {
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
}
