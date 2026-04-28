//!
//! Comparison and short-circuit logical expression lowering.
//!

use std::str::FromStr;

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::ir::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `sol.cmp` comparison.
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
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;
        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            self.state.builder.types.ui256
        };
        let lhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let predicate = Operator::from_str(operator)?.cmp_predicate();
        let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
        Ok((comparison, block))
    }

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
