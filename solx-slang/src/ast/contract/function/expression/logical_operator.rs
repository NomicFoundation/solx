//!
//! Short-circuit logical operator (`&&` / `||`), which lowers itself.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use solx_mlir::ods::sol::StoreOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

/// A short-circuit logical operator. Replaces a `default: bool` flag so `&&` and
/// `||` lower through one method that branches on the operator.
pub enum LogicalOperator {
    /// `&&` — evaluates the RHS only when the LHS is true.
    And,
    /// `||` — evaluates the RHS only when the LHS is false.
    Or,
}

impl LogicalOperator {
    /// The `i1` value the result keeps when the LHS short-circuits: `false` for
    /// `&&` (a false LHS makes the whole expression false), `true` for `||` (a
    /// true LHS makes it true).
    fn short_circuit_value(&self) -> bool {
        matches!(self, LogicalOperator::Or)
    }

    /// Emits this short-circuit operator via `sol.if` over an `i1` result slot,
    /// matching solc's pattern. The RHS is evaluated and stored only in the
    /// branch the LHS does NOT short-circuit — the `then` branch for `&&` (LHS
    /// true), the `else` branch for `||` (LHS false); the other branch keeps the
    /// short-circuit value.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit<'context, 'block>(
        self,
        emitter: &ExpressionContext<'_, 'context, 'block>,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let short_circuit_value = self.short_circuit_value();
        let BlockAnd { value: lhs, block } = left.emit(emitter, block)?;
        let lhs_bool = lhs.is_nonzero(&emitter.state.builder, &block).into_mlir();

        let i1_type = crate::ast::Type::signless(
            emitter.state.builder.context,
            solx_utils::BIT_LENGTH_BOOLEAN,
        )
        .into_mlir();
        let result_ptr = emitter.state.builder.emit_sol_alloca(i1_type, &block);
        let default_value =
            crate::ast::Value::boolean(short_circuit_value, &emitter.state.builder, &block)
                .into_mlir();
        sol_op_void!(
            &emitter.state.builder,
            &block,
            StoreOperation.val(default_value).addr(result_ptr)
        );

        let (then_block, else_block) = emitter.state.builder.emit_sol_if(lhs_bool, &block);
        let (rhs_block, short_circuit_block) = if short_circuit_value {
            (else_block, then_block)
        } else {
            (then_block, else_block)
        };

        // The non-short-circuiting branch evaluates the RHS and stores it.
        let BlockAnd {
            value: rhs,
            block: rhs_end,
        } = right.emit(emitter, rhs_block)?;
        let rhs_bool = rhs.is_nonzero(&emitter.state.builder, &rhs_end).into_mlir();
        sol_op_void!(
            &emitter.state.builder,
            &rhs_end,
            StoreOperation.val(rhs_bool).addr(result_ptr)
        );
        sol_op_void!(&emitter.state.builder, &rhs_end, YieldOperation.ins(&[]));
        // The short-circuiting branch keeps the default.
        sol_op_void!(
            &emitter.state.builder,
            &short_circuit_block,
            YieldOperation.ins(&[])
        );

        let result = emitter
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
        Ok((result, block))
    }
}
