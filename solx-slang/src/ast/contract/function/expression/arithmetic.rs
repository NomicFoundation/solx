//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a binary arithmetic Sol dialect operation.
    ///
    /// When `target_type` is `Some`, both operands are cast to that type and
    /// the result has that type (matching solc's type-annotated MLIR output).
    /// When `None`, selects the wider operand type by bit width.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: Operator,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = solx_mlir::TypeFactory::integer_bit_width(lhs.r#type());
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        // For exponentiation, the rhs (exponent) must be unsigned regardless
        // of the result type — `sol.exp` / `sol.cexp` require `AnyUnsignedInteger`.
        let rhs_target = if matches!(operator, Operator::Exponentiation) {
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            Type::from(IntegerType::unsigned(self.state.builder.context, rhs_width))
        } else {
            result_type
        };
        let rhs = TypeConversion::from_target_type(rhs_target, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.builder.context,
                self.state.builder.unknown_location,
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (old, _) = self.emit_increment_decrement(operand, operator, &block)?;
        Ok((old, block))
    }

    /// Emits prefix operators: `!`, `-`, `~`, `++`, `--`.
    ///
    /// When `target_type` is `Some`, unary operations use that type (matching
    /// solc's typed MLIR). When `None`, falls back to ui256 semantics.
    pub fn emit_prefix(
        &self,
        operator: Operator,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match operator {
            Operator::Increment | Operator::Decrement => {
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            Operator::BitwiseNot => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let result = block
                    .append_operation(
                        NotOperation::builder(
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                        )
                        .value(value)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.not always produces one result")
                    .into();
                Ok((result, block))
            }
            Operator::Not => {
                let (value, block) = self.emit_value(operand, block)?;
                let zero = self
                    .state
                    .builder
                    .emit_sol_constant(0, value.r#type(), &block);
                let cmp = self
                    .state
                    .builder
                    .emit_sol_cmp(value, zero, CmpPredicate::Eq, &block);
                let result_type = target_type.unwrap_or(self.state.builder.types.ui256);
                let result = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(cmp, &self.state.builder, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction. Checked negation
                // requires signed-type awareness (e.g. -INT_MIN should revert
                // in checked mode) which needs a dedicated op — not sol.csub,
                // since the operand may be in an unsigned literal type.
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                let zero = self
                    .state
                    .builder
                    .emit_sol_constant(0, operand_type, &block);
                let result = block
                    .append_operation(
                        SubOperation::builder(
                            self.state.builder.context,
                            self.state.builder.unknown_location,
                        )
                        .lhs(zero)
                        .rhs(value)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.sub always produces one result")
                    .into();
                Ok((result, block))
            }
            Operator::Delete => {
                // `delete m[k]` / `delete arr[i]` resets the indexed element.
                if let Expression::IndexAccessExpression(index_access) = operand {
                    let (address, element_type, block) =
                        self.emit_index_access_address(index_access, block)?;
                    let zero = self
                        .state
                        .builder
                        .emit_sol_constant(0, element_type, &block);
                    self.state
                        .builder
                        .emit_sol_store(zero, address, &block);
                    return Ok((zero, block));
                }
                // `delete x` resets `x` to its type's zero value. For the
                // experimental branch only scalar identifiers are supported —
                // reference-type deletion (arrays, mappings, structs) needs
                // dedicated lowering.
                let Expression::Identifier(identifier) = operand else {
                    anyhow::bail!("unsupported delete operand");
                };
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) =
                            self.environment.variable_with_type(&name);
                        let builder = &self.state.builder;
                        let zero = if melior::ir::r#type::IntegerType::try_from(element_type)
                            .is_ok()
                        {
                            builder.emit_sol_constant(0, element_type, &block)
                        } else if format!("{element_type}").starts_with("!sol.enum") {
                            let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
                            block
                                .append_operation(
                                    solx_mlir::ods::sol::EnumCastOperation::builder(
                                        builder.context,
                                        builder.unknown_location,
                                    )
                                    .inp(raw)
                                    .out(element_type)
                                    .build()
                                    .into(),
                                )
                                .result(0)
                                .expect("sol.enum_cast produces one result")
                                .into()
                        } else {
                            // Reference type variable (array/string/struct in
                            // memory). Real Solidity rebinds to a fresh empty
                            // instance, but our codegen has no analogue, so
                            // we bail rather than emit a misleading store.
                            anyhow::bail!(
                                "delete on a non-integer local '{name}' is not yet supported"
                            );
                        };
                        builder.emit_sol_store(zero, pointer, &block);
                        Ok((zero, block))
                    }
                    Some(Definition::StateVariable(state_variable)) => {
                        let (slot, byte_offset, location) = *self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .ok_or_else(|| {
                                anyhow::anyhow!("unregistered state variable: {name}")
                            })?;
                        let element_type = TypeConversion::resolve_state_variable_type(
                            &state_variable,
                            &self.state.builder,
                        )?;
                        // `sol.constant 0` requires an integer type. For
                        // enums, build a ui256 zero and bridge it to the
                        // enum type via `sol.enum_cast` before storing.
                        let builder = &self.state.builder;
                        let zero = if melior::ir::r#type::IntegerType::try_from(element_type)
                            .is_ok()
                        {
                            builder.emit_sol_constant(0, element_type, &block)
                        } else if format!("{element_type}").starts_with("!sol.enum") {
                            let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
                            block
                                .append_operation(
                                    solx_mlir::ods::sol::EnumCastOperation::builder(
                                        builder.context,
                                        builder.unknown_location,
                                    )
                                    .inp(raw)
                                    .out(element_type)
                                    .build()
                                    .into(),
                                )
                                .result(0)
                                .expect("sol.enum_cast produces one result")
                                .into()
                        } else {
                            // Reference / struct types — leave un-deleted for
                            // now (the test will fail with the old value).
                            builder.emit_sol_constant(0, builder.types.ui256, &block)
                        };
                        self.emit_storage_store(slot, byte_offset, zero, location, &block);
                        Ok((zero, block))
                    }
                    _ => anyhow::bail!("unsupported delete target: {name}"),
                }
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables via
    /// `resolve_to_definition()`.
    fn emit_increment_decrement(
        &self,
        operand: &Expression,
        operator: Operator,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
        // Unwrap parenthesised single-element tuples like `--(i)`.
        let unwrapped: Option<Expression> = match operand {
            Expression::TupleExpression(tuple) if tuple.items().len() == 1 => tuple
                .items()
                .iter()
                .next()
                .and_then(|item| item.expression()),
            _ => None,
        };
        let effective = unwrapped.as_ref().unwrap_or(operand);
        // Pointer-target inc/dec: `s.field++`, `arr[i]++`, `m[k]++`.
        if let Expression::MemberAccessExpression(access) = effective
            && let Some((address, element_type, block_after)) =
                self.emit_struct_field_address(access, *block)?
        {
            let block = block_after;
            let old = self
                .state
                .builder
                .emit_sol_load(address, element_type, &block)?;
            let one = self.state.builder.emit_sol_constant(1, element_type, &block);
            let new_value = block
                .append_operation(operator.emit_sol_binary_operation(
                    self.checked,
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                    old,
                    one,
                ))
                .result(0)
                .expect("binary operation always produces one result")
                .into();
            self.state
                .builder
                .emit_sol_store(new_value, address, &block);
            return Ok((old, new_value));
        }
        if let Expression::IndexAccessExpression(index_access) = effective {
            let (address, element_type, block_after) =
                self.emit_index_access_address(index_access, *block)?;
            let block = block_after;
            let old = self
                .state
                .builder
                .emit_sol_load(address, element_type, &block)?;
            let one = self.state.builder.emit_sol_constant(1, element_type, &block);
            let new_value = block
                .append_operation(operator.emit_sol_binary_operation(
                    self.checked,
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                    old,
                    one,
                ))
                .result(0)
                .expect("binary operation always produces one result")
                .into();
            self.state
                .builder
                .emit_sol_store(new_value, address, &block);
            return Ok((old, new_value));
        }

        let Expression::Identifier(identifier) = effective else {
            anyhow::bail!("unsupported operand for {operator:?}");
        };
        let name = identifier.name();

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let (slot, byte_offset, location) = *self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                )?;
                let old = self.emit_storage_load(slot, byte_offset, element_type, location, block)?;
                let one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.emit_storage_store(slot, byte_offset, new_value, location, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let typed_one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        typed_one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
