//!
//! Conditional (ternary `?:`) expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a ternary `cond ? a : b` to a `sol.if` over a stack result slot.
    ///
    /// Each branch evaluates its expression, casts it to the ternary's result
    /// type, and stores it into the slot; the slot is loaded afterwards.
    pub fn emit_conditional(
        &self,
        conditional: &ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = TypeConversion::resolve_slang_type(
            &conditional
                .get_type()
                .expect("binder types every ternary expression"),
            None,
            &self.state.builder,
        );

        let (condition_value, block) = self.emit_value(&conditional.operand(), block)?;
        let condition = self.emit_is_nonzero(condition_value, &block);

        let result_slot = self.state.builder.emit_sol_alloca(result_type, &block);
        let (then_block, else_block) = self.state.builder.emit_sol_if(condition, &block);
        self.emit_conditional_branch(
            &conditional.true_expression(),
            result_type,
            result_slot,
            then_block,
        )?;
        self.emit_conditional_branch(
            &conditional.false_expression(),
            result_type,
            result_slot,
            else_block,
        )?;

        let result = self
            .state
            .builder
            .emit_sol_load(result_slot, result_type, &block)?;
        Ok((result, block))
    }

    /// Evaluates a ternary branch, casts it to the result type, stores it into
    /// the shared result slot, and yields.
    fn emit_conditional_branch(
        &self,
        expression: &Expression,
        result_type: Type<'context>,
        result_slot: Value<'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let (value, end) = self.emit_value(expression, block)?;
        let cast = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &end,
        );
        self.state.builder.emit_sol_store(cast, result_slot, &end);
        self.state.builder.emit_sol_yield(&end);
        Ok(())
    }
}
