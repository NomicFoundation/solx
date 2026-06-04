//!
//! Return statement lowering to `sol.return`.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ReturnStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `return` statement to `sol.return`.
    pub fn emit_return(
        &self,
        _return_statement: &ReturnStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("return statement")
    }
}
