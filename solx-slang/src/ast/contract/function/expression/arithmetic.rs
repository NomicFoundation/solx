//!
//! Arithmetic expression lowering: binary additive, multiplicative, and
//! exponentiation operations.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::operator_binding::OperatorBindings;
use crate::ast::type_conversion::TypeConversion;

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
        // A binary operator on a user-defined value type bound via
        // `using {f as op} for T global;` dispatches to the bound function
        // (which carries its own checked/unchecked context), not native
        // arithmetic. Operands are evaluated left-to-right, as Solidity requires.
        if let Some(function_id) = self.user_defined_binary_operator(left, operator) {
            let (lhs, block) = self.emit_value(left, block)?;
            let (rhs, block) = self.emit_value(right, block)?;
            let result = self.emit_operator_call(function_id, vec![lhs, rhs], &block)?;
            return Ok((result, block));
        }

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
        let value = self.emit_value_binary_operation(operator, lhs, rhs, result_type, &block);
        Ok((value, block))
    }

    /// Emits a binary `operator` over already-materialized `lhs`/`rhs` values,
    /// producing a value of `result_type`. Shared by [`Self::emit_binary_op`]
    /// (the expression path) and the compound-assignment path so both get the
    /// fixed-bytes bitwise bridge.
    ///
    /// `sol.and`/`or`/`xor`/`shl`/`shr` are integer-only, but Solidity allows
    /// them on `bytesN` / `byte` (bitwise on the raw bytes). Bridge the fixed-
    /// bytes operand(s) through the equivalent unsigned integer `ui(8*N)` and
    /// cast the result back. A shift amount is a plain integer, so on a shift
    /// only the shifted value is bridged.
    pub fn emit_value_binary_operation(
        &self,
        operator: Operator,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        let is_shift = matches!(operator, Operator::ShiftLeft | Operator::ShiftRight);
        let is_bitwise = is_shift
            || matches!(
                operator,
                Operator::BitwiseAnd | Operator::BitwiseOr | Operator::BitwiseXor
            );

        // Prepare both operands and, when bridged, the type to restore the
        // result to. `sol.and`/`or`/`xor`/`shl`/`shr` are integer-only, but
        // Solidity allows them on `bytesN` / `byte`: bridge each fixed-bytes
        // operand through the equivalent unsigned integer `ui(8*N)` and restore
        // the result. A shift amount is already a plain integer, so on a shift
        // only the shifted value is bridged. Every other operation runs on
        // operands coerced straight to `result_type`.
        let (lhs, rhs, restore_type) = if is_bitwise
            && let Some(width) = solx_mlir::TypeFactory::fixed_bytes_or_byte_width(result_type)
        {
            let int_type = Type::from(IntegerType::unsigned(builder.context, 8 * width));
            let lhs_fb = TypeConversion::coerce(lhs, result_type, builder, block);
            let lhs = builder.emit_sol_cast(lhs_fb, int_type, block);
            let rhs = if is_shift {
                TypeConversion::coerce(rhs, int_type, builder, block)
            } else {
                let rhs_fb = TypeConversion::coerce(rhs, result_type, builder, block);
                builder.emit_sol_cast(rhs_fb, int_type, block)
            };
            (lhs, rhs, Some(result_type))
        } else {
            let lhs = TypeConversion::coerce(lhs, result_type, builder, block);
            // `**` keeps its exponent its own (unsigned) type: `sol.exp`/`sol.cexp`
            // take an unsigned exponent of any width alongside a possibly-signed
            // base, so the exponent must NOT be coerced to the (signed) result
            // type the way a symmetric operator's operands are. (solc: `cexp
            // si256, ui8`.)
            let rhs = if matches!(operator, Operator::Exponentiation) {
                rhs
            } else {
                TypeConversion::coerce(rhs, result_type, builder, block)
            };
            (lhs, rhs, None)
        };

        let result = block
            .append_operation(operator.emit_sol_binary_operation(
                self.arithmetic_mode,
                builder,
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();

        match restore_type {
            Some(fixed) => builder.emit_sol_cast(result, fixed, block),
            None => result,
        }
    }

    /// The function bound to `operator` for `left`'s user-defined value type via
    /// `using {f as op} for T global;`, or `None` when `left` is not such a type
    /// or the operator carries no binding. A pure classification — the caller
    /// emits the dispatched call.
    fn user_defined_binary_operator(
        &self,
        left: &Expression,
        operator: Operator,
    ) -> Option<NodeId> {
        let user_operator = OperatorBindings::binary_operator(operator)?;
        self.user_defined_operator(left, user_operator)
    }
}
