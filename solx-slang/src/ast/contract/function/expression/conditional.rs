//!
//! Conditional (ternary) expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ConditionalExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a conditional expression `c ? a : b`.
    ///
    /// Allocates a result slot, branches on the condition with `sol.if`, and
    /// stores each arm's value (cast to the result type) into the slot.
    pub fn emit_conditional(
        &self,
        conditional: &ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = self
            .resolve_slang_type(conditional.get_type())
            .unwrap_or(self.state.builder.types.ui256);
        let condition = conditional.operand();
        let (condition_value, block) = self.emit_value(&condition, block)?;
        let condition_boolean = self.emit_is_nonzero(condition_value, &block);

        let result_slot = self.state.builder.emit_sol_alloca(result_type, &block);
        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        let true_expression = conditional.true_expression();
        let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
        let then_cast = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            then_value,
            &self.state.builder,
            &then_end,
        );
        self.state
            .builder
            .emit_sol_store(then_cast, result_slot, &then_end);
        self.state.builder.emit_sol_yield(&then_end);

        let false_expression = conditional.false_expression();
        let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
        let else_cast = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            else_value,
            &self.state.builder,
            &else_end,
        );
        self.state
            .builder
            .emit_sol_store(else_cast, result_slot, &else_end);
        self.state.builder.emit_sol_yield(&else_end);

        let result = self
            .state
            .builder
            .emit_sol_load(result_slot, result_type, &block)?;
        Ok((result, block))
    }
}
