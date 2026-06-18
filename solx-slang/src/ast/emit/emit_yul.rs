//!
//! The inline-assembly (Yul) emission trait: each Yul node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::statement::assembly::YulContext;

/// Emits an inline-assembly (Yul) node to MLIR, appending operations to `block`.
///
/// Implemented per node directly on the Slang AST type (the orphan rule forbids an
/// inherent method). The context is `&mut YulContext` — a Yul `let` declares
/// variables. `Output` stays associated because the Yul family is not uniform and,
/// unlike a statement, a Yul node never diverges solx control flow: a Yul statement
/// yields its continuation `BlockRef` (not an `Option`), an expression its
/// `(word, continuation)` pair, a function call its `(words, continuation)`.
///
/// `'context: 'block` because a produced word cannot outlive its block; `'state`
/// (the emitter's field borrows) is a method parameter, appearing only in the
/// threaded `&mut YulContext`. Emission never fails — slang validates the source
/// beforehand, so an unsupported construct panics rather than erroring.
pub trait EmitYul<'context: 'block, 'block> {
    /// The node's result: a continuation, or a value (or values) paired with one.
    type Output;

    /// Emits this Yul node into `block`.
    fn emit<'state>(
        &self,
        context: &mut YulContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output;
}
