//!
//! Short-circuit logical expression lowering (`&&`, `||`, `!`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;

use super::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers short-circuit `&&`: seed an `i1` slot with `false`, and only when
    /// the left operand is true evaluate the right and store it.
    pub(super) fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_short_circuit(left, right, false, block)
    }

    /// Lowers short-circuit `||`: seed an `i1` slot with `true`, and only when
    /// the left operand is false evaluate the right and store it.
    pub(super) fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_short_circuit(left, right, true, block)
    }

    /// Lowers `!operand` as a comparison against zero (`operand == 0`).
    pub(super) fn emit_not(
        &self,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_value(operand, block)?;
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), &block);
        let result = self
            .state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Eq, &block);
        Ok((result, block))
    }

    /// Emits a short-circuit `&&`/`||` via a `sol.if` over an `i1` slot.
    ///
    /// `seed` is the result when the left operand short-circuits: `false` for
    /// `&&` (left false ⇒ false), `true` for `||` (left true ⇒ true). The right
    /// operand is evaluated only on the non-short-circuiting branch.
    fn emit_short_circuit(
        &self,
        left: &Expression,
        right: &Expression,
        seed: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (left_value, block) = self.emit_value(left, block)?;
        let left_boolean = self.emit_is_nonzero(left_value, &block);

        let i1_type = self.state.builder.types.i1;
        let result_slot = self.state.builder.emit_sol_alloca(i1_type, &block);
        let seed_value = self.state.builder.emit_bool(seed, &block);
        self.state
            .builder
            .emit_sol_store(seed_value, result_slot, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(left_boolean, &block);
        // `&&` evaluates the right operand when the left is true (then branch);
        // `||` when the left is false (else branch). `seed` selects the side.
        let evaluation_block = if seed { else_block } else { then_block };
        let short_circuit_block = if seed { then_block } else { else_block };

        let (right_value, evaluation_end) = self.emit_value(right, evaluation_block)?;
        let right_boolean = self.emit_is_nonzero(right_value, &evaluation_end);
        self.state
            .builder
            .emit_sol_store(right_boolean, result_slot, &evaluation_end);
        self.state.builder.emit_sol_yield(&evaluation_end);
        self.state.builder.emit_sol_yield(&short_circuit_block);

        let result = self
            .state
            .builder
            .emit_sol_load(result_slot, i1_type, &block)?;
        Ok((result, block))
    }
}
