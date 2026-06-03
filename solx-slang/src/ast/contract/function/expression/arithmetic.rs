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
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::operator_binding;

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
        // A binary operator on a user-defined-value-type operand bound via
        // `using {f as op} for T global;` dispatches to the bound function
        // (which carries its own checked context), not native arithmetic.
        if let Some(result) =
            self.try_emit_user_defined_binary_operator(left, right, operator, block)?
        {
            return Ok(result);
        }

        // Solidity evaluates subexpressions left-to-right; `emit_binary_operands`
        // preserves that order while materializing a string literal paired with
        // a `bytesN` / `byte` operand as a fixedbytes/byte constant.
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = solx_mlir::TypeFactory::integer_bit_width(lhs.r#type());
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });

        let value = self.emit_value_binary_operation(operator, lhs, rhs, result_type, &block);
        Ok((value, block))
    }

    /// Emits a binary `operator` over already-materialized `lhs`/`rhs` values,
    /// producing a value of `result_type`. Shared by [`Self::emit_binary_op`]
    /// (the expression path) and the compound-assignment path so both get the
    /// fixed-bytes bitwise bridge below.
    ///
    /// `sol.and/or/xor/shl/shr` are integer-only, but Solidity allows them on
    /// `bytesN` / `byte` (bitwise on the raw bytes). Bridge the fixed-bytes
    /// operand(s) through the equivalent unsigned integer `ui(8*N)` and cast
    /// the result back. Shift amounts are plain integers, so only the shifted
    /// value is bridged on the rhs.
    pub fn emit_value_binary_operation(
        &self,
        operator: Operator,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let is_shift = matches!(operator, Operator::ShiftLeft | Operator::ShiftRight);
        let is_bitwise =
            is_shift || matches!(operator, Operator::BitwiseAnd | Operator::BitwiseOr | Operator::BitwiseXor);
        if is_bitwise
            && let Some(width) = solx_mlir::TypeFactory::fixed_bytes_width(result_type)
                .or_else(|| solx_mlir::TypeFactory::is_sol_byte(result_type).then_some(1))
        {
            let builder = &self.state.builder;
            let int_type = Type::from(IntegerType::unsigned(builder.context, 8 * width));
            let lhs_fb = TypeConversion::from_target_type(result_type, builder).emit(lhs, builder, block);
            let lhs_int = builder.emit_sol_cast(lhs_fb, int_type, block);
            let rhs_int = if is_shift {
                // The shift amount is an integer; resize it to the bridge width.
                TypeConversion::from_target_type(int_type, builder).emit(rhs, builder, block)
            } else {
                let rhs_fb = TypeConversion::from_target_type(result_type, builder).emit(rhs, builder, block);
                builder.emit_sol_cast(rhs_fb, int_type, block)
            };
            let result_int: Value<'context, 'block> = block
                .append_operation(operator.emit_sol_binary_operation(
                    self.checked,
                    builder.context,
                    builder.unknown_location,
                    lhs_int,
                    rhs_int,
                ))
                .result(0)
                .expect("binary operation always produces one result")
                .into();
            return builder.emit_sol_cast(result_int, result_type, block);
        }

        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            block,
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
            block,
        );
        block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.builder.context,
                self.state.builder.unknown_location,
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
    }

    /// Dispatches a binary operation to a user-defined operator function when
    /// the left operand is a user-defined value type with a binding for
    /// `operator` (`using {f as op} for T global;`). Returns `None` when no such
    /// binding applies, leaving the caller to emit native arithmetic.
    ///
    /// The bound function is a free function carrying its own checked/unchecked
    /// context, so this correctly reproduces e.g. an `unchecked` operator body
    /// invoked from a checked caller.
    fn try_emit_user_defined_binary_operator(
        &self,
        left: &Expression,
        right: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        let Some(user_operator) = operator_binding::binary_operator(operator) else {
            return Ok(None);
        };
        let Some(SlangType::UserDefinedValue(udvt_type)) = left.get_type() else {
            return Ok(None);
        };
        let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition() else {
            return Ok(None);
        };
        let Some(&function_id) = self
            .state
            .operator_bindings
            .get(&(udvt_definition.node_id(), user_operator))
        else {
            return Ok(None);
        };

        let (lhs, rhs, block) = self.emit_binary_operands(left, right, block)?;
        let result = self.emit_operator_call(function_id, vec![lhs, rhs], &block)?;
        Ok(Some((result, block)))
    }

    /// Dispatches a prefix operation to a user-defined operator function when
    /// the operand is a user-defined value type with a binding for `operator`
    /// (a unary `using {f as -}` / `using {f as ~}`). Returns `None` when no
    /// such binding applies, leaving the caller to emit native negation.
    fn try_emit_user_defined_unary_operator(
        &self,
        operator: Operator,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        let Some(user_operator) = operator_binding::unary_operator(operator) else {
            return Ok(None);
        };
        let Some(SlangType::UserDefinedValue(udvt_type)) = operand.get_type() else {
            return Ok(None);
        };
        let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition() else {
            return Ok(None);
        };
        let Some(&function_id) = self
            .state
            .operator_bindings
            .get(&(udvt_definition.node_id(), user_operator))
        else {
            return Ok(None);
        };

        let (value, block) = self.emit_value(operand, block)?;
        let result = self.emit_operator_call(function_id, vec![value], &block)?;
        Ok(Some((result, block)))
    }

    /// Resolves the bound operator function `function_id` and emits a call to it
    /// with the already-evaluated `argument_values`, each coerced to its
    /// parameter type. Returns the operator's single result value.
    fn emit_operator_call(
        &self,
        function_id: NodeId,
        mut argument_values: Vec<Value<'context, 'block>>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let (mlir_name, parameter_types, return_types) =
            self.state.resolve_function(function_id)?;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            *value = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                *value,
                &self.state.builder,
                block,
            );
        }
        let results = self.state.builder.emit_sol_call_results(
            mlir_name,
            &argument_values,
            return_types,
            block,
        )?;
        Ok(results
            .into_iter()
            .next()
            .expect("a user-defined operator returns one value"))
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
        // A `-`/`~` prefix on a user-defined-value-type operand bound via
        // `using {f as -} for T global;` dispatches to the bound function.
        if let Some(result) = self.try_emit_user_defined_unary_operator(operator, operand, block)? {
            return Ok(result);
        }

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
                // `sol.not` is integer-only; for `bytesN` / `byte` bridge through
                // the equivalent unsigned integer `ui(8*N)` and cast back.
                let builder = &self.state.builder;
                let fixed_width = solx_mlir::TypeFactory::fixed_bytes_width(operand_type)
                    .or_else(|| solx_mlir::TypeFactory::is_sol_byte(operand_type).then_some(1));
                let (value, restore_type) = match fixed_width {
                    Some(width) => {
                        let int_type = Type::from(IntegerType::unsigned(builder.context, 8 * width));
                        (builder.emit_sol_cast(value, int_type, &block), Some(operand_type))
                    }
                    None => (value, None),
                };
                let result: Value<'context, 'block> = block
                    .append_operation(
                        NotOperation::builder(builder.context, builder.unknown_location)
                            .value(value)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.not always produces one result")
                    .into();
                let result = match restore_type {
                    Some(fixed) => builder.emit_sol_cast(result, fixed, &block),
                    None => result,
                };
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
            Operator::Delete => self.emit_delete(operand, block),
            _ => unimplemented!("unsupported prefix operator: {operator:?}"),
        }
    }

    /// Emits `delete <operand>`, resetting the target to its type's zero value.
    /// Indexed elements (`delete m[k]` / `delete arr[i]`) and struct fields
    /// (`delete s.field`) reset in place; a bare identifier dispatches to the
    /// local-variable or state-variable handler.
    fn emit_delete(
        &self,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // `delete m[k]` / `delete arr[i]` resets the indexed element.
        if let Expression::IndexAccessExpression(index_access) = operand {
            let (address, element_type, block) =
                self.emit_index_access_address(index_access, block)?;
            let zero = self.state.builder.emit_sol_constant(0, element_type, &block);
            self.state.builder.emit_sol_store(zero, address, &block);
            return Ok((zero, block));
        }
        // `delete s.field` resets the addressed struct field: scalars store
        // zero, enums store their zero variant, and reference-typed fields
        // (nested arrays / structs / bytes in storage) recurse through
        // `sol.delete`.
        if let Expression::MemberAccessExpression(access) = operand
            && let Some((address, element_type, block)) =
                self.emit_struct_field_address(access, block)?
        {
            let builder = &self.state.builder;
            if solx_mlir::TypeFactory::is_sol_reference(element_type) {
                builder.emit_sol_delete(address, &block);
                let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                return Ok((placeholder, block));
            }
            let zero = if melior::ir::r#type::IntegerType::try_from(element_type).is_ok() {
                builder.emit_sol_constant(0, element_type, &block)
            } else if solx_mlir::TypeFactory::is_sol_enum(element_type) {
                let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
                builder.emit_sol_enum_cast(raw, element_type, &block)
            } else {
                builder.emit_sol_constant(0, element_type, &block)
            };
            builder.emit_sol_store(zero, address, &block);
            return Ok((zero, block));
        }
        // `delete x` resets `x` to its type's zero value; reference-type
        // deletion needs storage-class-specific lowering, so dispatch on the
        // resolved definition.
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("delete of a non-identifier operand");
        };
        let name = identifier.name();
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.emit_delete_local_variable(&name, identifier.get_type(), block)
            }
            Some(Definition::StateVariable(state_variable)) => {
                self.emit_delete_state_variable(&state_variable, block)
            }
            _ => unimplemented!("unsupported delete target: {name}"),
        }
    }

    /// Emits `delete x` for a local variable / parameter, rebinding it to its
    /// type's zero value: integers to `0`, enums to their zero variant,
    /// function pointers to the default pointer, dynamic aggregates (arrays /
    /// `bytes` / `string`) to a fresh empty allocation, and fixed aggregates
    /// (structs / fixed arrays) to a zero-initialised allocation.
    fn emit_delete_local_variable(
        &self,
        name: &str,
        slang_type: Option<SlangType>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (pointer, element_type) = self.environment.variable_with_type(name);
        let builder = &self.state.builder;
        let zero = if melior::ir::r#type::IntegerType::try_from(element_type).is_ok() {
            builder.emit_sol_constant(0, element_type, &block)
        } else if solx_mlir::TypeFactory::is_sol_enum(element_type) {
            let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
            builder.emit_sol_enum_cast(raw, element_type, &block)
        } else {
            // Reference-typed local: `delete` rebinds it to a fresh zero value.
            // Function pointers reset to the default (zero) pointer; dynamic
            // aggregates (arrays / `bytes` / `string`) to a fresh empty
            // allocation (length 0); fixed aggregates (structs / fixed arrays)
            // to a zero-initialised allocation.
            match slang_type {
                Some(SlangType::Function(_)) => {
                    builder.emit_sol_default_func_constant(element_type, &block)
                }
                Some(SlangType::Array(_) | SlangType::String(_) | SlangType::Bytes(_)) => {
                    let zero_size = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    builder.emit_sol_malloc_sized(element_type, zero_size, &block)
                }
                Some(SlangType::FixedSizeArray(_) | SlangType::Struct(_)) => {
                    builder.emit_sol_malloc(element_type, &block)
                }
                _ => unimplemented!(
                    "delete on a non-integer local '{name}' is not yet supported"
                ),
            }
        };
        builder.emit_sol_store(zero, pointer, &block);
        Ok((zero, block))
    }

    /// Emits `delete x` for a state variable, resetting its storage to the
    /// default. Reference types (array / struct / `string` / `bytes`) reset by
    /// copying a fresh zero-initialised aggregate or recursing through
    /// `sol.delete`; `delete` on a mapping is a no-op; value types and enums
    /// store a zero word (enums bridged through `sol.enum_cast`).
    fn emit_delete_state_variable(
        &self,
        state_variable: &slang_solidity_v2::ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (slot, byte_offset, location) = *self
            .storage_layout
            .get(&state_variable.node_id())
            .expect("unregistered state variable");
        let element_type =
            TypeConversion::resolve_state_variable_type(state_variable, &self.state.builder)?;
        // `sol.constant 0` requires an integer type. For enums, build a ui256
        // zero and bridge it to the enum type via `sol.enum_cast` before storing.
        let builder = &self.state.builder;

        // `delete x` on a reference-typed storage variable (array / struct /
        // string / bytes) resets it to its default by copying a freshly
        // allocated, zero-initialised memory aggregate of the same type into the
        // slot. `sol.copy` writes the storage destination and clears the previous
        // tail for dynamic aggregates. `delete` on a mapping is a no-op.
        if solx_mlir::TypeFactory::is_sol_reference(element_type) {
            let declared_type = state_variable.get_type().ok_or_else(|| {
                anyhow::anyhow!("unresolved type for state variable")
            })?;
            match &declared_type {
                // `delete` on a mapping is a no-op in Solidity.
                SlangType::Mapping(_) => {
                    let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    return Ok((placeholder, block));
                }
                // Dynamic `bytes`/`string` reset to empty: copy a freshly
                // allocated zero-length memory buffer into the slot. `sol.copy`
                // writes the storage destination and clears the previous tail.
                SlangType::Bytes(_) | SlangType::String(_) => {
                    let memory_type = TypeConversion::resolve_slang_type(
                        &declared_type,
                        Some(solx_utils::DataLocation::Memory),
                        builder,
                    );
                    let zero_size = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    let default_value =
                        builder.emit_sol_malloc_sized(memory_type, zero_size, &block);
                    let address = builder.emit_sol_addr_of(
                        &crate::ast::contract::ContractEmitter::storage_symbol(
                            slot,
                            byte_offset,
                            location,
                        ),
                        Self::address_type(builder, element_type, location, &declared_type),
                        &block,
                    );
                    builder.emit_sol_copy(default_value, address, &block);
                    let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    return Ok((placeholder, block));
                }
                // Arrays and structs: `sol.delete` recursively clears every
                // storage slot the aggregate occupies (matching solc's deep
                // delete — dynamic members reset to empty, nested aggregates
                // recurse).
                SlangType::Struct(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => {
                    let address = builder.emit_sol_addr_of(
                        &crate::ast::contract::ContractEmitter::storage_symbol(
                            slot,
                            byte_offset,
                            location,
                        ),
                        Self::address_type(builder, element_type, location, &declared_type),
                        &block,
                    );
                    builder.emit_sol_delete(address, &block);
                    let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    return Ok((placeholder, block));
                }
                _ => unimplemented!(
                    "delete of a reference-type storage variable is not yet supported"
                ),
            }
        }

        let zero = if melior::ir::r#type::IntegerType::try_from(element_type).is_ok() {
            builder.emit_sol_constant(0, element_type, &block)
        } else if solx_mlir::TypeFactory::is_sol_enum(element_type) {
            let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
            builder.emit_sol_enum_cast(raw, element_type, &block)
        } else if solx_mlir::TypeFactory::is_sol_reference(element_type) {
            // Array / struct / string / bytes / mapping need recursive zeroing,
            // not yet implemented. Fail cleanly rather than storing a
            // type-mismatched `ui256(0)` into a reference-typed slot (which the
            // verifier rejects).
            unimplemented!("delete of a reference-type storage variable is not yet supported");
        } else {
            // Word-sized types (e.g. function pointers) zero correctly with a
            // plain `ui256(0)` store.
            builder.emit_sol_constant(0, builder.types.ui256, &block)
        };
        self.emit_storage_store(slot, byte_offset, zero, location, &block);
        Ok((zero, block))
    }

    /// Computes `old + 1` / `old - 1` for a `++` / `--`, typed to
    /// `element_type` and respecting the emitter's checked-arithmetic mode.
    fn emit_inc_dec_step(
        &self,
        old: Value<'context, 'block>,
        element_type: Type<'context>,
        operator: Operator,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let one = self.state.builder.emit_sol_constant(1, element_type, block);
        block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.builder.context,
                self.state.builder.unknown_location,
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
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
            let new_value = self.emit_inc_dec_step(old, element_type, operator, &block);
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
            let new_value = self.emit_inc_dec_step(old, element_type, operator, &block);
            self.state
                .builder
                .emit_sol_store(new_value, address, &block);
            return Ok((old, new_value));
        }

        let Expression::Identifier(identifier) = effective else {
            unimplemented!("unsupported operand for {operator:?}");
        };
        let name = identifier.name();

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let (slot, byte_offset, location) = *self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("unregistered state variable");
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                )?;
                let old = self.emit_storage_load(slot, byte_offset, element_type, location, block)?;
                let new_value = self.emit_inc_dec_step(old, element_type, operator, block);
                self.emit_storage_store(slot, byte_offset, new_value, location, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let new_value = self.emit_inc_dec_step(old, element_type, operator, block);
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => unreachable!("unresolved identifier: {name}"),
            Some(_) => unimplemented!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
