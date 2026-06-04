//!
//! Event emit statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::EmitStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers an `emit` statement to `sol.emit`.
    pub fn emit_event(
        &self,
        _emit_statement: &EmitStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("event emit")
    }
}
