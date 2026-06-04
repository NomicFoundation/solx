//!
//! Expression statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an expression statement: the expression is emitted for its side
    /// effects and any value it yields is discarded.
    ///
    /// The call form of `revert` (`revert()` / `revert("msg")`) is a statement
    /// that never yields a value, so it is routed to its dedicated lowering.
    pub fn emit_expression_statement(
        &self,
        statement: &ExpressionStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = statement.expression();
        if let Expression::FunctionCallExpression(call) = &expression
            && let Expression::Identifier(identifier) = call.operand()
            && matches!(identifier.resolve_to_built_in(), Some(BuiltIn::Revert))
        {
            return self.emit_revert_call(call, block);
        }

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (_value, block) = emitter.emit(&expression, block)?;
        Ok(Some(block))
    }
}
