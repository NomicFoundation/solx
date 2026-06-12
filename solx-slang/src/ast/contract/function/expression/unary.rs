//!
//! Unary expression lowering: prefix and postfix operators, increment/decrement.
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

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::operator_binding::OperatorBindings;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Some((old, _new, block)) =
            self.emit_increment_decrement_indexed(operand, operator, block)?
        {
            return Ok((old, block));
        }
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
        // A `-` / `~` prefix on a user-defined value type bound via
        // `using {f as op} for T global;` dispatches to the bound function
        // rather than emitting native negation / bitwise-not.
        if let Some(function_id) = self.user_defined_unary_operator(operand, operator) {
            let (value, block) = self.emit_value(operand, block)?;
            let result = self.emit_operator_call(function_id, vec![value], &block)?;
            return Ok((result, block));
        }

        match operator {
            Operator::Increment | Operator::Decrement => {
                if let Some((_old, new_value, block)) =
                    self.emit_increment_decrement_indexed(operand, operator, block)?
                {
                    return Ok((new_value, block));
                }
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            Operator::BitwiseNot => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value =
                    TypeConversion::coerce(value, operand_type, &self.state.builder, &block);
                // `sol.not` is integer-only; for a `bytesN` / `byte` operand
                // bridge through the equivalent unsigned integer `ui(8*N)` and
                // cast the result back to the fixed-bytes type.
                let builder = &self.state.builder;
                let (value, restore_type) =
                    match solx_mlir::TypeFactory::fixed_bytes_or_byte_width(operand_type) {
                        Some(width) => {
                            let int_type =
                                Type::from(IntegerType::unsigned(builder.context, 8 * width));
                            (
                                builder.emit_sol_cast(value, int_type, &block),
                                Some(operand_type),
                            )
                        }
                        None => (value, None),
                    };
                let result: Value<'context, 'block> =
                    sol_op!(builder, block, NotOperation.value(value));
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
                let result = TypeConversion::coerce(cmp, result_type, &self.state.builder, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction. Checked negation
                // requires signed-type awareness (e.g. -INT_MIN should revert
                // in checked mode) which needs a dedicated op — not sol.csub,
                // since the operand may be in an unsigned literal type.
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value =
                    TypeConversion::coerce(value, operand_type, &self.state.builder, &block);
                let zero = self
                    .state
                    .builder
                    .emit_sol_constant(0, operand_type, &block);
                let result = sol_op!(
                    &self.state.builder,
                    block,
                    SubOperation.lhs(zero).rhs(value)
                );
                Ok((result, block))
            }
            _ => unimplemented!("unsupported prefix operator: {operator:?}"),
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
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("unsupported operand for {operator:?}");
        };

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .unwrap_or_else(|| {
                        unimplemented!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                )?;
                let old = self.emit_storage_load(slot, element_type, block)?;
                let new_value = self.emit_step(old, operator, element_type, block);
                self.emit_storage_store(slot, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                let (pointer, element_type) =
                    self.environment.variable_with_type(definition.node_id());
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let new_value = self.emit_step(old, operator, element_type, block);
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => unreachable!("slang resolves every identifier reference"),
            Some(other) => {
                unimplemented!(
                    "unsupported operand for {operator:?}: {:?}",
                    other.node_id()
                )
            }
        }
    }

    /// Emits `++` / `--` on a *computed* lvalue — an `a[i]` element or a struct
    /// field — returning `Some((old, new, block))`, or `None` for any other
    /// operand so the caller falls through to the identifier path.
    ///
    /// Unlike the identifier case, the operand evaluates a sub-expression (the
    /// index / base), so the post-evaluation block is threaded back out. The
    /// resolved address is a value-typed pointer (a reference type is never the
    /// operand of `++`/`--`), so it is loaded, stepped, and stored like the
    /// pointer lvalue in `emit_assignment`.
    fn emit_increment_decrement_indexed(
        &self,
        operand: &Expression,
        operator: Operator,
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
                self.emit_index_access_address(index_access, block)?
            }
            Expression::MemberAccessExpression(access) => {
                self.emit_struct_field_address(access, block)?
            }
            _ => return Ok(None),
        };
        let old = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        let new_value = self.emit_step(old, operator, element_type, &block);
        self.state
            .builder
            .emit_sol_store(new_value, address, &block);
        Ok(Some((old, new_value, block)))
    }

    /// Applies the `++` / `--` step to a loaded value: adds or subtracts a typed
    /// `1` through the operator's binary operation, honoring the arithmetic
    /// mode. The two lvalue kinds (storage slot and stack pointer) share this
    /// step; only the surrounding load/store differs.
    fn emit_step(
        &self,
        old: Value<'context, 'block>,
        operator: Operator,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let one = self.state.builder.emit_sol_constant(1, element_type, block);
        block
            .append_operation(operator.emit_sol_binary_operation(
                self.arithmetic_mode,
                &self.state.builder,
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
    }

    /// The function bound to the prefix `operator` for `operand`'s user-defined
    /// value type via `using {f as op} for T global;`, or `None` when `operand`
    /// is not such a type or the operator carries no binding. A pure
    /// classification — the caller emits the dispatched call.
    fn user_defined_unary_operator(
        &self,
        operand: &Expression,
        operator: Operator,
    ) -> Option<NodeId> {
        let user_operator = OperatorBindings::unary_operator(operator)?;
        self.user_defined_operator(operand, user_operator)
    }
}
