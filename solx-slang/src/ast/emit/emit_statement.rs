//!
//! The statement emission trait: each Slang statement node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::statement::StatementContext;

/// Emits a Slang statement node to MLIR, appending operations to `block` and
/// threading the continuation block back to the caller.
///
/// Implemented per node directly on the Slang AST type (the orphan rule forbids an
/// inherent method). The context is `&mut StatementContext` — a statement may
/// declare variables and open blocks, so it needs `&mut`, the one difference from
/// the shared-`&` expression family. The result is uniform across every statement,
/// so it is a fixed `Option<BlockRef>` rather than an associated type: `Some` is the
/// continuation block, `None` when control diverged (`return` / `break` /
/// `continue`).
///
/// `'context: 'block` because the returned `BlockRef` cannot outlive its block;
/// `'state` (the emitter's field borrows) is a method parameter, appearing only in
/// the threaded `&mut StatementContext`. Emission never fails — slang validates the
/// source beforehand, so an unsupported construct panics rather than erroring.
pub trait EmitStatement<'context: 'block, 'block> {
    /// Emits this statement into `block`, returning the continuation.
    fn emit<'state>(
        &self,
        context: &mut StatementContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>>;
}
