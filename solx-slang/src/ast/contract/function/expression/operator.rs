//!
//! Solidity operator, bridged from slang's typed per-expression operator enums.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;
use melior::ir::r#type::IntegerType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::UserDefinedOperator;
use solx_mlir::VariableBinding;
use solx_mlir::ods::sol::AddOperation;
use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::CAddOperation;
use solx_mlir::ods::sol::CDivOperation;
use solx_mlir::ods::sol::CExpOperation;
use solx_mlir::ods::sol::CMulOperation;
use solx_mlir::ods::sol::CSubOperation;
use solx_mlir::ods::sol::DivOperation;
use solx_mlir::ods::sol::ExpOperation;
use solx_mlir::ods::sol::ModOperation;
use solx_mlir::ods::sol::MulOperation;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::StoreOperation;
use solx_mlir::ods::sol::SubOperation;
use solx_mlir::ods::sol::XorOperation;

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::operator_binding::OperatorBindings;
use crate::ast::type_conversion::TypeConversion;

/// Solidity operator, bridged from slang's typed per-expression operator enums
/// (`AdditiveExpressionOperator`, `ShiftExpressionOperator`, …) — never parsed
/// from source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    // ---- Arithmetic ----
    /// `+` (binary)
    Add,
    /// `-` (binary or unary negation)
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponentiation,

    // ---- Bitwise ----
    /// `&`
    BitwiseAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `<<`
    ShiftLeft,
    /// `>>` (and the no-op `>>>`)
    ShiftRight,
    /// `~`
    BitwiseNot,

    // ---- Logical ----
    /// `!`
    Not,

    // ---- Step ----
    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl Operator {
    /// The function bound to `user_operator` for `operand`'s user-defined value
    /// type via `using {f as op} for T global;`, or `None` when `operand` is not
    /// such a type or the operator carries no binding. The shared UDVT lookup
    /// behind the binary ([`Self::emit_binary`]) and unary ([`Self::emit_prefix`])
    /// operator dispatch.
    fn user_defined_operator<'context, 'block>(
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        user_operator: UserDefinedOperator,
    ) -> Option<NodeId> {
        let SlangType::UserDefinedValue(udvt_type) = operand.get_type()? else {
            return None;
        };
        let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition() else {
            return None;
        };
        context
            .state
            .operator_bindings
            .get(&(udvt_definition.node_id(), user_operator))
            .copied()
    }

    /// Calls the bound user-defined-operator function `function_id` with the
    /// already-evaluated `argument_values`, each coerced to its parameter type,
    /// and returns the operator's single result value. Shared by the binary
    /// ([`Self::emit_binary`]) and unary ([`Self::emit_prefix`]) operator dispatch.
    fn emit_operator_call<'context, 'block>(
        context: &ExpressionContext<'_, 'context, 'block>,
        function_id: NodeId,
        argument_values: Vec<crate::ast::Value<'context, 'block>>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let (mlir_name, parameter_types, return_types) =
            context.state.resolve_function(function_id)?;
        let argument_values: Vec<_> = argument_values
            .into_iter()
            .zip(parameter_types)
            .map(|(value, &parameter_type)| {
                value
                    .coerce_to(parameter_type, &context.state.builder, block)
                    .into_mlir()
            })
            .collect();
        let results = context.state.builder.emit_sol_call_results(
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

    /// Builds a Sol dialect binary operation via ODS-generated builders.
    ///
    /// In [`ArithmeticMode::Checked`] mode, uses checked variants (`sol.cadd`,
    /// `sol.csub`, `sol.cmul`, `sol.cdiv`, `sol.cexp`) for arithmetic operators.
    /// Modulo, bitwise, and shift operators are always unchecked. Result type is
    /// inferred from `lhs` (`SameOperandsAndResultType`).
    ///
    /// # Panics
    ///
    /// Panics if called on a unary-only operator (`Not` / `BitwiseNot`), which
    /// the prefix emitter handles instead.
    pub fn emit_sol_binary_operation<'context>(
        self,
        mode: ArithmeticMode,
        builder: &Builder<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        let checked = matches!(mode, ArithmeticMode::Checked);
        match self {
            Self::Add | Self::Increment if checked => {
                sol_op_build!(builder, CAddOperation.lhs(lhs).rhs(rhs))
            }
            Self::Add | Self::Increment => sol_op_build!(builder, AddOperation.lhs(lhs).rhs(rhs)),
            Self::Subtract | Self::Decrement if checked => {
                sol_op_build!(builder, CSubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Subtract | Self::Decrement => {
                sol_op_build!(builder, SubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Multiply if checked => sol_op_build!(builder, CMulOperation.lhs(lhs).rhs(rhs)),
            Self::Multiply => sol_op_build!(builder, MulOperation.lhs(lhs).rhs(rhs)),
            Self::Divide if checked => sol_op_build!(builder, CDivOperation.lhs(lhs).rhs(rhs)),
            Self::Divide => sol_op_build!(builder, DivOperation.lhs(lhs).rhs(rhs)),
            Self::Remainder => sol_op_build!(builder, ModOperation.lhs(lhs).rhs(rhs)),
            Self::Exponentiation if checked => {
                sol_op_build!(
                    builder,
                    CExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs)
                )
            }
            Self::Exponentiation => {
                sol_op_build!(builder, ExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::BitwiseAnd => sol_op_build!(builder, AndOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseOr => sol_op_build!(builder, OrOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseXor => sol_op_build!(builder, XorOperation.lhs(lhs).rhs(rhs)),
            // `sol.shl`/`sol.shr` now accept a `bytesN` (or integer) value with an
            // independent integer shift amount (`AllTypesMatch<lhs, result>`, rhs
            // free), so the result type is no longer inferable from both operands
            // and must be set explicitly — it follows the shifted value (`lhs`).
            Self::ShiftLeft => {
                sol_op_build!(builder, ShlOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::ShiftRight => {
                sol_op_build!(builder, ShrOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            _ => unreachable!(
                "emit_sol_binary_operation called on non-arithmetic operator: {self:?}"
            ),
        }
    }

    /// Lowers a binary expression `left <op> right` to its result value.
    ///
    /// A user-defined value type bound via `using {f as op} for T global;`
    /// dispatches to the bound function (which carries its own
    /// checked/unchecked context), with operands evaluated left-to-right as
    /// Solidity requires. Otherwise both operands are materialised and combined
    /// by [`Self::emit_value_binary`]. With no `target_type`, the wider operand
    /// type (by bit width) is selected.
    pub fn emit_binary<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        left: &Expression,
        right: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        crate::ast::Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        if let Some(function_id) = OperatorBindings::binary_operator(self)
            .and_then(|user_operator| Self::user_defined_operator(context, left, user_operator))
        {
            let BlockAnd { value: lhs, block } = left.emit(context, block)?;
            let BlockAnd { value: rhs, block } = right.emit(context, block)?;
            let result = Self::emit_operator_call(context, function_id, vec![lhs, rhs], &block)?;
            return Ok((result.into(), block));
        }

        let BlockAnd { value: rhs, block } = right.emit(context, block)?;
        let BlockAnd { value: lhs, block } = left.emit(context, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = solx_mlir::TypeFactory::integer_bit_width(lhs.r#type());
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let value = self.emit_value_binary(
            context.arithmetic_mode,
            &context.state.builder,
            lhs,
            rhs,
            result_type,
            &block,
        );
        Ok((value, block))
    }

    /// Combines already-materialised `lhs`/`rhs` into a value of `result_type`.
    /// Shared by [`Self::emit_binary`] (the expression path) and the
    /// compound-assignment path, so both get the fixed-bytes bitwise bridge.
    ///
    /// `sol.and`/`or`/`xor`/`shl`/`shr` are integer-only, but Solidity allows
    /// them on `bytesN` / `byte` (bitwise on the raw bytes). Bridge the fixed-
    /// bytes operand(s) through the equivalent unsigned integer `ui(8*N)` and
    /// cast the result back. A shift amount is a plain integer, so on a shift
    /// only the shifted value is bridged.
    pub fn emit_value_binary<'context, 'block>(
        self,
        mode: ArithmeticMode,
        builder: &Builder<'context>,
        lhs: crate::ast::Value<'context, 'block>,
        rhs: crate::ast::Value<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> crate::ast::Value<'context, 'block> {
        let is_shift = matches!(self, Operator::ShiftLeft | Operator::ShiftRight);
        let is_bitwise = is_shift
            || matches!(
                self,
                Operator::BitwiseAnd | Operator::BitwiseOr | Operator::BitwiseXor
            );

        let (lhs, rhs, restore_type) = if is_bitwise
            && let Some(width) = solx_mlir::TypeFactory::fixed_bytes_or_byte_width(result_type)
        {
            let int_type = Type::from(IntegerType::unsigned(builder.context, 8 * width));
            let lhs = lhs
                .coerce_to(result_type, builder, block)
                .cast(int_type, builder, block)
                .into_mlir();
            let rhs = if is_shift {
                rhs.coerce_to(int_type, builder, block).into_mlir()
            } else {
                rhs.coerce_to(result_type, builder, block)
                    .cast(int_type, builder, block)
                    .into_mlir()
            };
            (lhs, rhs, Some(result_type))
        } else {
            let lhs = lhs.coerce_to(result_type, builder, block).into_mlir();
            // `**` keeps its exponent its own (unsigned) type: `sol.exp`/`sol.cexp`
            // take an unsigned exponent of any width alongside a possibly-signed
            // base, so the exponent must NOT be coerced to the (signed) result
            // type the way a symmetric operator's operands are. (solc: `cexp
            // si256, ui8`.)
            let rhs = if matches!(self, Operator::Exponentiation) {
                rhs.into_mlir()
            } else {
                rhs.coerce_to(result_type, builder, block).into_mlir()
            };
            (lhs, rhs, None)
        };

        let result: Value<'context, 'block> = block
            .append_operation(self.emit_sol_binary_operation(mode, builder, lhs, rhs))
            .result(0)
            .expect("binary operation always produces one result")
            .into();

        match restore_type {
            Some(fixed) => crate::ast::Value::from(result).cast(fixed, builder, block),
            None => result.into(),
        }
    }

    /// Emits postfix `++` / `--`, returning the old value.
    pub fn emit_postfix<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        crate::ast::Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        if let Some((old, _new, block)) =
            self.emit_increment_decrement_indexed(context, operand, block)?
        {
            return Ok((old.into(), block));
        }
        let (old, _) = self.emit_increment_decrement(context, operand, &block)?;
        Ok((old.into(), block))
    }

    /// Emits prefix operators: `!`, `-`, `~`, `++`, `--`.
    ///
    /// A `-` / `~` prefix on a user-defined value type bound via
    /// `using {f as op} for T global;` dispatches to the bound function rather
    /// than emitting native negation / bitwise-not. When `target_type` is `Some`,
    /// the operation uses that type (matching solc's typed MLIR); otherwise it
    /// falls back to the operand's own type / ui256.
    pub fn emit_prefix<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        crate::ast::Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        if let Some(function_id) = OperatorBindings::unary_operator(self)
            .and_then(|user_operator| Self::user_defined_operator(context, operand, user_operator))
        {
            let BlockAnd { value, block } = operand.emit(context, block)?;
            let result = Self::emit_operator_call(context, function_id, vec![value], &block)?;
            return Ok((result.into(), block));
        }

        match self {
            Operator::Increment | Operator::Decrement => {
                if let Some((_old, new_value, block)) =
                    self.emit_increment_decrement_indexed(context, operand, block)?
                {
                    return Ok((new_value.into(), block));
                }
                let (_old, new_value) = self.emit_increment_decrement(context, operand, &block)?;
                Ok((new_value.into(), block))
            }
            Operator::BitwiseNot => {
                let BlockAnd { value, block } = operand.emit(context, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = value.coerce_to(operand_type, &context.state.builder, &block);
                // `sol.not` is integer-only; for a `bytesN` / `byte` operand
                // bridge through the equivalent unsigned integer `ui(8*N)` and
                // cast the result back to the fixed-bytes type.
                let builder = &context.state.builder;
                let (value, restore_type) =
                    match solx_mlir::TypeFactory::fixed_bytes_or_byte_width(operand_type) {
                        Some(width) => {
                            let int_type =
                                Type::from(IntegerType::unsigned(builder.context, 8 * width));
                            (
                                value.cast(int_type, builder, &block).into_mlir(),
                                Some(operand_type),
                            )
                        }
                        None => (value.into_mlir(), None),
                    };
                let result: Value<'context, 'block> =
                    sol_op!(builder, block, NotOperation.value(value));
                let result = match restore_type {
                    Some(fixed) => crate::ast::Value::from(result)
                        .cast(fixed, builder, &block)
                        .into_mlir(),
                    None => result,
                };
                Ok((result.into(), block))
            }
            Operator::Not => {
                let BlockAnd { value, block } = operand.emit(context, block)?;
                let zero = context
                    .state
                    .builder
                    .emit_sol_constant(0, value.r#type(), &block);
                let cmp = value.compare(
                    crate::ast::Value::from(zero),
                    CmpPredicate::Eq,
                    &context.state.builder,
                    &block,
                );
                let result_type = target_type.unwrap_or(context.state.builder.types.ui256);
                let result = cmp.coerce_to(result_type, &context.state.builder, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction. Checked negation
                // requires signed-type awareness (e.g. -INT_MIN should revert
                // in checked mode) which needs a dedicated op — not sol.csub,
                // since the operand may be in an unsigned literal type.
                let BlockAnd { value, block } = operand.emit(context, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = value
                    .coerce_to(operand_type, &context.state.builder, &block)
                    .into_mlir();
                let zero = context
                    .state
                    .builder
                    .emit_sol_constant(0, operand_type, &block);
                let result: Value<'context, 'block> = sol_op!(
                    &context.state.builder,
                    block,
                    SubOperation.lhs(zero).rhs(value)
                );
                Ok((result.into(), block))
            }
            _ => unimplemented!("unsupported prefix operator: {self:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)` for an
    /// identifier lvalue (a local / parameter or a state variable).
    fn emit_increment_decrement<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("unsupported operand for {self:?}");
        };

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = context
                    .storage_layout
                    .get(&state_variable.node_id())
                    .unwrap_or_else(|| {
                        unimplemented!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &context.state.builder,
                )?;
                let old = slot.load(&context.state.builder, element_type, block)?;
                let new_value = self.emit_step(context, old, element_type, block);
                slot.store(&context.state.builder, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                let VariableBinding {
                    pointer,
                    element_type,
                } = context.environment.variable_with_type(definition.node_id());
                let old = context
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let new_value = self.emit_step(context, old, element_type, block);
                sol_op_void!(
                    &context.state.builder,
                    block,
                    StoreOperation.val(new_value).addr(pointer)
                );
                Ok((old, new_value))
            }
            None => unreachable!("slang resolves every identifier reference"),
            Some(other) => {
                unimplemented!("unsupported operand for {self:?}: {:?}", other.node_id())
            }
        }
    }

    /// Emits `++` / `--` on a *computed* lvalue — an `a[i]` element or a struct
    /// field — returning `Some((old, new, block))`, or `None` for any other
    /// operand so the caller falls through to the identifier path.
    fn emit_increment_decrement_indexed<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            Value<'context, 'block>,
            Value<'context, 'block>,
            BlockRef<'context, 'block>,
        )>,
    > {
        let (address, element_type, block) = match operand {
            Expression::IndexAccessExpression(index_access) => {
                context.emit_index_access_address(index_access, block)?
            }
            Expression::MemberAccessExpression(access) => {
                context.emit_struct_field_address(access, block)?
            }
            _ => return Ok(None),
        };
        let old = context
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        let new_value = self.emit_step(context, old, element_type, &block);
        sol_op_void!(
            &context.state.builder,
            &block,
            StoreOperation.val(new_value).addr(address)
        );
        Ok(Some((old, new_value, block)))
    }

    /// Applies the `++` / `--` step to a loaded value: adds or subtracts a typed
    /// `1` through this operator's binary operation, honoring the arithmetic
    /// mode. Both lvalue kinds (storage slot and stack pointer) share this step.
    fn emit_step<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        old: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let one = context
            .state
            .builder
            .emit_sol_constant(1, element_type, block);
        block
            .append_operation(self.emit_sol_binary_operation(
                context.arithmetic_mode,
                &context.state.builder,
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
    }
}
