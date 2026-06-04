//!
//! Expression statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::contract::function::statement::revert;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits an expression used as a statement, discarding its value.
    ///
    /// A bare `revert(...)` / `require(...)`-style call to the `revert`
    /// built-in is routed to revert lowering; any other expression is emitted
    /// for its side effects.
    pub fn emit_expression_statement(
        &mut self,
        expression_statement: &ExpressionStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = expression_statement.expression();
        if let Expression::FunctionCallExpression(call) = &expression
            && let Expression::Identifier(identifier) = call.operand()
            && identifier.name() == revert::IDENTIFIER
        {
            return self.emit_revert_call(call, block);
        }
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (_, block) = emitter.emit(&expression, block)?;
        Ok(Some(block))
    }
}
