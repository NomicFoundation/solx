//!
//! The lowering trait: each Slang AST node emits its own MLIR.
//!

pub mod block_and;

pub use self::block_and::BlockAnd;

use melior::ir::BlockRef;

/// Lowers a Slang AST node to MLIR, appending operations to `block` and threading
/// the continuation back to the caller.
///
/// Implemented per node (one node, one module) through the extension-trait
/// pattern. The associated `Context` carries the shared lowering state —
/// `&ExpressionContext` (an expression cannot declare variables) or
/// `&mut StatementContext` (a statement can) — so the `&`/`&mut` split *is* that
/// invariant. `Output` is the node family's result: a [`BlockAnd`] for an
/// expression, an `Option<BlockRef>` continuation for a statement (`None` when
/// control diverged).
///
/// Four lifetimes, outermost → innermost: `'context` (MLIR context), `'block`
/// (block region), `'state` (the emitter's own field borrows), `'scope` (the
/// borrow of the emitter passed to `emit`). `'state` and `'scope` are distinct
/// because a `&mut` context is invariant — the dispatcher's short borrow of the
/// emitter (`'scope`) cannot be unified with the emitter's longer field-borrow
/// lifetime (`'state`).
pub trait Emit<'context, 'block, 'state, 'scope> {
    /// The shared lowering state threaded into emission.
    type Context;
    /// The emission result.
    type Output;

    /// Emits this node into `block`.
    ///
    /// # Errors
    ///
    /// Returns an error if the node contains unsupported constructs.
    fn emit(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Self::Output>;
}
