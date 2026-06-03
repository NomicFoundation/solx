//!
//! Expression statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ExpressionStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;

use super::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an expression statement: the expression is emitted for its side
    /// effects and any value it yields is discarded.
    pub(super) fn emit_expression_statement(
        &self,
        statement: &ExpressionStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (_value, block) = emitter.emit(&statement.expression(), block)?;
        Ok(Some(block))
    }
}
