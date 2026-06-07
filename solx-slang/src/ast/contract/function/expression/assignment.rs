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
                    self.arithmetic_mode,
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

        let result = self.store_into_target(&target, value, &block);
        Ok((result, block))
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
                        let declared_type = state_variable
                            .get_type()
                            .expect("slang types every state variable");
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .unwrap_or_else(|| {
                                unimplemented!(
                                    "unregistered state variable {:?}",
                                    state_variable.node_id()
                                )
                            })
                            .clone();
                        let element_type = TypeConversion::resolve_slang_type(
                            &declared_type,
                            None,
                            &self.state.builder,
                        );
                        // Reference-typed storage (fixed/dynamic arrays, `string`,
                        // `bytes`, structs) is assigned by copying the RHS
                        // reference's contents into the slot via `sol.copy`, just
                        // like a reference-typed inline initializer; a whole
                        // mapping is not assignable. Value-typed storage stores
                        // the coerced scalar directly.
                        if declared_type.is_reference_type()
                            && !matches!(declared_type, ast::Type::Mapping(_))
                        {
                            let address_type = Self::address_type(
                                &self.state.builder,
                                element_type,
                                slot.location,
                                &declared_type,
                            );
                            let storage_ref = self.state.builder.emit_sol_addr_of(
                                &slot.name,
                                address_type,
                                &block,
                            );
                            AssignmentTarget::ReferenceCopy(storage_ref)
                        } else {
                            AssignmentTarget::Storage(slot, element_type)
                        }
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
        let lhs_items = tuple.items();

        let (values, block) = match right {
            Expression::TupleExpression(rhs_tuple) => {
                let rhs_items = rhs_tuple.items();
                assert!(
                    rhs_items.len() == lhs_items.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} RHS values",
                    lhs_items.len(),
                    rhs_items.len(),
                );
                let mut values = Vec::with_capacity(rhs_items.len());
                let mut current = block;
                for item in rhs_items.iter() {
                    let inner = item
                        .expression()
                        .expect("a tuple assignment RHS element has an inner expression");
                    let (value, next) = self.emit_value(&inner, current)?;
                    values.push(value);
                    current = next;
                }
                (values, current)
            }
            Expression::FunctionCallExpression(call) => {
                let call_emitter = CallEmitter::new(self);
                let (values, current) = call_emitter.emit_function_call_results(call, block)?;
                assert!(
                    values.len() == lhs_items.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} call results",
                    lhs_items.len(),
                    values.len(),
                );
                (values, current)
            }
            _ => unimplemented!(
                "tuple assignment with this right-hand side shape is not yet supported"
            ),
        };

        let mut targets = Vec::with_capacity(lhs_items.len());
        let mut block = block;
        for item in lhs_items.iter() {
            match item.expression() {
                Some(lvalue) => {
                    let (target, next) = self.resolve_assignment_target(&lvalue, block)?;
                    targets.push(Some(target));
                    block = next;
                }
                None => targets.push(None),
            }
        }

        let mut result = None;
        for (target, value) in targets.iter().zip(values).rev() {
            if let Some(target) = target {
                result = Some(self.store_into_target(target, value, &block));
            }
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
