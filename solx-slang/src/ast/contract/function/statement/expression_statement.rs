//!
//! Expression statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ExpressionStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an expression statement, discarding any produced value.
    pub fn emit_expression_statement(
        &self,
        _statement: &ExpressionStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("expression statement")
    }
}
