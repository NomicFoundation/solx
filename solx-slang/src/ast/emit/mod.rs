//!
//! The emission trait: each Slang AST node emits its own MLIR.
//!

pub mod address;
pub mod block_and;
pub mod place;

use melior::ir::BlockRef;

pub use self::address::EmitAddress;
pub use self::block_and::BlockAnd;
pub use self::place::Place;

/// Emits a Slang AST node to MLIR, appending operations to `block` and threading
/// the continuation back to the caller.
///
/// Implemented per node (one node, one module). The associated `Context` carries
/// the shared emission scope — `&ExpressionContext` (an expression cannot declare
/// variables) or `&mut StatementContext` (a statement can) — so the `&`/`&mut`
/// split *is* that invariant. `Output` is the node family's result: a [`BlockAnd`]
/// for an expression, an `Option<BlockRef>` continuation for a statement (`None`
/// when control diverged).
///
/// Four lifetimes, outermost → innermost: `'context` (MLIR context), `'block`
/// (block region), `'state` (the emitter's own field borrows), `'scope` (the
/// borrow of the emitter passed to `emit`). `'state` and `'scope` are distinct
/// because a `&mut` context is invariant — the dispatcher's short borrow of the
/// emitter (`'scope`) cannot be unified with the emitter's longer field-borrow
/// lifetime (`'state`).
///
/// Emission never fails: slang validates the source beforehand, so a node always
/// produces its value, and an unsupported construct or violated invariant panics
/// instead of returning an error.
pub trait Emit<'context, 'block, 'state, 'scope> {
    /// The shared emission scope (expression or statement) threaded into `emit`.
    type Context;
    /// The node family's result.
    type Output;

    /// Emits this node into `block`.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output;
}
