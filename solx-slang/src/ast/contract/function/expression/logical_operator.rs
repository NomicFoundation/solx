//!
//! Short-circuit logical operator (`&&` / `||`), which emits itself.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_expression::EmitExpression;

/// A short-circuit logical operator (`&&` / `||`).
pub enum LogicalOperator {
    /// `&&`: evaluates the RHS only when the LHS is true.
    And,
    /// `||`: evaluates the RHS only when the LHS is false.
    Or,
}

impl LogicalOperator {
    /// The `i1` value the result keeps when the LHS short-circuits: `false` for `&&`, `true` for `||`.
    fn short_circuit_value(&self) -> bool {
        matches!(self, LogicalOperator::Or)
    }

    /// Emits this short-circuit operator via `sol.if` over an `i1` result slot. The RHS is evaluated
    /// only in the branch the LHS does NOT short-circuit; the other branch keeps the short-circuit value.
    pub fn emit<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let short_circuit_value = self.short_circuit_value();
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let lhs_bool = lhs.is_nonzero(context.state, &block).into_mlir();

        let i1_type = AstType::signless(context.state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN);
        let result_ptr = Pointer::stack(i1_type, context.state, &block);
        let default_value = AstValue::boolean(short_circuit_value, context.state, &block);
        result_ptr.store(default_value, context.state, &block);

        let (then_block, else_block) = mlir_region_op!(
            context.state, &block,
            IfOperation.cond(lhs_bool); then_region, else_region
        );
        let (rhs_block, short_circuit_block) = if short_circuit_value {
            (else_block, then_block)
        } else {
            (then_block, else_block)
        };

        let BlockAnd {
            value: rhs,
            block: rhs_end,
        } = right.emit(context, rhs_block);
        let rhs_bool = rhs.is_nonzero(context.state, &rhs_end);
        result_ptr.store(rhs_bool, context.state, &rhs_end);
        mlir_op_void!(context.state, &rhs_end, YieldOperation.ins(&[]));
        mlir_op_void!(context.state, &short_circuit_block, YieldOperation.ins(&[]));

        let result = result_ptr.load(i1_type, context.state, &block).into_mlir();
        (result, block)
    }
}
