//!
//! Arithmetic expression lowering: binary additive, multiplicative, and
//! exponentiation operations.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
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
        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.arithmetic_mode,
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
