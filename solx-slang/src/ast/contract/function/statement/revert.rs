//!
//! Revert statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::RevertStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `revert` statement to `sol.revert`.
    pub fn emit_revert(
        &self,
        _revert: &RevertStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("revert statement")
    }
}
