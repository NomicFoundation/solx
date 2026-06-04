//!
//! Short-circuit logical expression lowering (`&&`, `||`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits short-circuit `&&` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `false`, and only evaluate the RHS when the LHS is true.
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
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type = self.state.builder.types.i1;
        let result_ptr = self.state.builder.emit_sol_alloca(i1_type, &block);
        let false_value = self.state.builder.emit_bool(false, &block);
        self.state
            .builder
            .emit_sol_store(false_value, result_ptr, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(lhs_bool, &block);

        // Then: LHS was true — evaluate RHS and store result.
        let (rhs, then_end) = self.emit_value(right, then_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        self.state
            .builder
            .emit_sol_store(rhs_bool, result_ptr, &then_end);
        self.state.builder.emit_sol_yield(&then_end);

        // Else: LHS was false — result stays false.
        self.state.builder.emit_sol_yield(&else_block);

        let result = self
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
        Ok((result, block))
    }

    /// Emits short-circuit `||` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `true`, and only evaluate the RHS when the LHS is false.
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
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type = self.state.builder.types.i1;
        let result_ptr = self.state.builder.emit_sol_alloca(i1_type, &block);
        let true_value = self.state.builder.emit_bool(true, &block);
        self.state
            .builder
            .emit_sol_store(true_value, result_ptr, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(lhs_bool, &block);

        // Then: LHS was true — result stays true.
        self.state.builder.emit_sol_yield(&then_block);

        // Else: LHS was false — evaluate RHS and store result.
        let (rhs, else_end) = self.emit_value(right, else_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        self.state
            .builder
            .emit_sol_store(rhs_bool, result_ptr, &else_end);
        self.state.builder.emit_sol_yield(&else_end);

        let result = self
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
        Ok((result, block))
    }
}
