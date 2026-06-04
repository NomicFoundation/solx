//!
//! Local variable declaration statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::VariableDeclarationStatement;

use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a local variable declaration statement.
    pub fn emit_variable_declaration(
        &mut self,
        _declaration: &VariableDeclarationStatement,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        unimplemented!("variable declaration")
    }
}
