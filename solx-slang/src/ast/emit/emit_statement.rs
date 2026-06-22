//!
//! The statement emission trait: each Slang statement node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::statement::StatementContext;

/// Emits a Slang statement node to MLIR, threading the continuation block back to the caller.
///
/// Implemented per node directly on the Slang AST type (the orphan rule forbids an inherent method).
/// The context is `&mut StatementContext` (a statement may declare variables). The result is a fixed
/// `Option<BlockRef>`: `Some` continuation, `None` when control diverged (`return` / `break` / `continue`).
pub trait EmitStatement<'context: 'block, 'block> {
    /// Emits this statement into `block`, returning the continuation.
    fn emit<'state>(
        &self,
        context: &mut StatementContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>>;
}
