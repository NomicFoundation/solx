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
    /// State variable — storage slot, byte offset within the slot, and
    /// declared element type.
    Storage(U256, u32, Type<'context>),
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
            let (value, block) = self.emit_value(&assign.right_operand(), block)?;
            let result = self.store_to_lvalue(target, value, &block);
            return Ok((result, block));
        }

        // Compound assignment (`+=`, `-=`, ...) reads the current value, applies
        // the operator, and writes back — so it needs a value-typed, loadable
        // target. Reference types are never the LHS of a compound operator.
        let LvalueTarget::Store(store_target) = target else {
            anyhow::bail!("compound assignment to a reference-typed lvalue is not supported");
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
            AssignmentTarget::Storage(slot, byte_offset, element_type) => {
                let old = self.emit_storage_load(*slot, *byte_offset, *element_type, &block)?;
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
                        let declared_type = state_variable.get_type().ok_or_else(|| {
                            anyhow::anyhow!("unresolved type for state variable: {name}")
                        })?;
                        let &(slot, byte_offset) = self
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
                        // Reference-typed storage (fixed/dynamic arrays,
                        // `string`, `bytes`, structs) is assigned by copying the
                        // RHS reference into the slot. Mappings are not assignable.
                        if declared_type.is_reference_type()
                            && !matches!(declared_type, slang_solidity_v2::ast::Type::Mapping(_))
                        {
                            let address_type = Self::address_type(
                                &self.state.builder,
                                element_type,
                                solx_utils::DataLocation::Storage,
                                &declared_type,
                            );
                            let storage_ref = self.state.builder.emit_sol_addr_of(
                                &crate::ast::contract::ContractEmitter::storage_symbol(
                                    slot, byte_offset,
                                ),
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
                            )),
                            block,
                        ))
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
                let (address, element_type, block) = self
                    .emit_struct_field_address(access, block)?
                    .expect("slang validates a member-access lvalue resolves to a struct field");
                if address.r#type() == element_type {
                    // Reference-typed struct field addressed by reference.
                    Ok((LvalueTarget::ReferenceCopy(address), block))
                } else {
                    Ok((
                        LvalueTarget::Store(AssignmentTarget::Pointer(address, element_type)),
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
            LvalueTarget::Store(AssignmentTarget::Storage(slot, byte_offset, element_type)) => {
                let stored_value = TypeConversion::from_target_type(element_type, &self.state.builder)
                    .emit(value, &self.state.builder, block);
                self.emit_storage_store(slot, byte_offset, stored_value, block);
                stored_value
            }
        }
    }

    /// Emits a tuple / destructuring assignment `(a, b, ...) = rhs`.
    ///
    /// Solidity evaluates the entire right-hand side before performing any
    /// assignment (so e.g. `(a, b) = (b, a)` swaps), so all RHS values are
    /// materialised first, then stored into the LHS components left to right.
    /// Blank components (`(, b) = ...`) discard their value.
    fn emit_tuple_assignment(
        &self,
        tuple: &slang_solidity_v2::ast::TupleExpression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let items = tuple.items();
        let (values, mut block) = match right {
            Expression::TupleExpression(rhs_tuple) => {
                let rhs_items = rhs_tuple.items();
                anyhow::ensure!(
                    rhs_items.len() == items.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} RHS values",
                    items.len(),
                    rhs_items.len(),
                );
                let mut values = Vec::with_capacity(rhs_items.len());
                let mut current = block;
                for item in rhs_items.iter() {
                    let inner = item.expression().ok_or_else(|| {
                        anyhow::anyhow!("empty tuple element on RHS of assignment")
                    })?;
                    let (value, next) = self.emit_value(&inner, current)?;
                    values.push(value);
                    current = next;
                }
                (values, current)
            }
            Expression::FunctionCallExpression(call) => {
                let call_emitter = CallEmitter::new(self);
                let (values, current) = call_emitter.emit_function_call_results(call, block)?;
                anyhow::ensure!(
                    values.len() == items.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} call results",
                    items.len(),
                    values.len(),
                );
                (values, current)
            }
            _ => anyhow::bail!("tuple assignment with this right-hand side shape is not yet supported"),
        };

        let mut last_stored = None;
        for (item, value) in items.iter().zip(values) {
            let Some(lvalue) = item.expression() else {
                // Blank slot (`(, b) = ...`): the value is discarded.
                continue;
            };
            let (target, next) = self.resolve_lvalue(&lvalue, block)?;
            block = next;
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
}
