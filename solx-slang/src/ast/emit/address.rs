//!
//! The lvalue emission trait: an assignable expression emits its address.
//!

use melior::ir::BlockRef;

use crate::ast::emit::BlockAnd;
use crate::ast::emit::Place;

/// Emits the [`Place`] an assignable expression denotes (its `!sol.ptr` and
/// element type), without the trailing `sol.load` / `sol.store`. A readable
/// place's value [`Emit`](crate::ast::Emit) loads from it and an assignment
/// stores into it, so the address is computed once here and shared by both. The
/// `Context` is the shared `&ExpressionContext` an expression `Emit` threads — an
/// lvalue declares no variables.
pub trait EmitAddress<'context, 'block, 'state, 'scope> {
    /// The shared emission scope threaded into `emit_address`.
    type Context;

    /// Emits this expression's place into `block`.
    fn emit_address(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>>;
}
