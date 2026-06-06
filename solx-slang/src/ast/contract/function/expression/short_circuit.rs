//!
//! Short-circuit logical expression lowering: `&&` and `||` (`sol.if` regions).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits short-circuit `&&`: evaluates the RHS only when the LHS is true.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_short_circuit(left, right, false, block)
    }

    /// Emits short-circuit `||`: evaluates the RHS only when the LHS is false.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_short_circuit(left, right, true, block)
    }

    /// Emits a short-circuit logical operator via `sol.if` over an `i1` result
    /// slot, matching solc's pattern. `default` is the value the result keeps
    /// when the LHS short-circuits (`false` for `&&`, `true` for `||`); the RHS
    /// is evaluated and stored only in the branch the LHS does NOT short-circuit
    /// — the `then` branch for `&&` (LHS true), the `else` branch for `||` (LHS
    /// false).
    fn emit_short_circuit(
        &self,
        left: &Expression,
        right: &Expression,
        default: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type = self.state.builder.types.i1;
        let result_ptr = self.state.builder.emit_sol_alloca(i1_type, &block);
        let default_value = self.state.builder.emit_bool(default, &block);
        self.state
            .builder
            .emit_sol_store(default_value, result_ptr, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(lhs_bool, &block);
        let (rhs_block, short_circuit_block) = if default {
            (else_block, then_block)
        } else {
            (then_block, else_block)
        };

        // The non-short-circuiting branch evaluates the RHS and stores it.
        let (rhs, rhs_end) = self.emit_value(right, rhs_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &rhs_end);
        self.state
            .builder
            .emit_sol_store(rhs_bool, result_ptr, &rhs_end);
        self.state.builder.emit_sol_yield(&rhs_end);
        // The short-circuiting branch keeps the default.
        self.state.builder.emit_sol_yield(&short_circuit_block);

        let result = self
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
        Ok((result, block))
    }
}
