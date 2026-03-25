//!
//! Comparison and short-circuit logical expression lowering.
//!

use std::str::FromStr;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `sol.cmp` comparison, cast to `ui256` via `sol.cast`.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        let predicate = Operator::from_str(operator)?.cmp_predicate();
        let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
        let value = self
            .state
            .builder
            .emit_sol_cast_to_ui256(comparison, &block);
        Ok((value, block))
    }

    /// Emits short-circuit `&&` using value-producing `scf.if`.
    ///
    /// Result is always a canonical boolean (0 or 1).
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
        let (lhs, block) = self.emit(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
        let (then_block, else_block, result) =
            self.state.builder.emit_scf_if(lhs_bool, ui256, &block)?;

        // Then: LHS was true — evaluate RHS and yield normalized result.
        let (rhs, then_end) = self.emit(right, then_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        let rhs_normalized = self
            .state
            .builder
            .emit_sol_cast_to_ui256(rhs_bool, &then_end);
        self.state
            .builder
            .emit_scf_yield(&[rhs_normalized], &then_end);

        // Else: LHS was false — yield 0.
        let zero = self.state.builder.emit_sol_constant(0, &else_block);
        self.state.builder.emit_scf_yield(&[zero], &else_block);

        Ok((result, block))
    }

    /// Emits short-circuit `||` using value-producing `scf.if`.
    ///
    /// Result is always a canonical boolean (0 or 1).
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
        let (lhs, block) = self.emit(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
        let (then_block, else_block, result) =
            self.state.builder.emit_scf_if(lhs_bool, ui256, &block)?;

        // Then: LHS was true — yield 1.
        let one = self.state.builder.emit_sol_constant(1, &then_block);
        self.state.builder.emit_scf_yield(&[one], &then_block);

        // Else: LHS was false — evaluate RHS and yield normalized result.
        let (rhs, else_end) = self.emit(right, else_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        let rhs_normalized = self
            .state
            .builder
            .emit_sol_cast_to_ui256(rhs_bool, &else_end);
        self.state
            .builder
            .emit_scf_yield(&[rhs_normalized], &else_end);

        Ok((result, block))
    }
}
