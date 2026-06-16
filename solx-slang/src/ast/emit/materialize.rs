//!
//! Target-typed string-literal materialisation: a string literal emits as a
//! fixed-bytes / byte constant when used where `bytesN` / `byte` is expected.
//!

use melior::ir::BlockRef;
use melior::ir::Type;

use crate::ast::emit::BlockAnd;

/// Emits a string literal toward an expected MLIR type. The one case a string
/// literal's natural [`Emit`](crate::ast::Emit) is wrong: toward `bytesN` /
/// `byte` it is a compile-time, left-aligned fixed-bytes / byte constant — the
/// literal occupies the high bytes, zero-padded on the right — not a runtime
/// `sol.string` (which the integer-only verifier rejects). slang types the
/// literal `Literal(String)` context-free, so the target reaches the literal only
/// from the use site. For any other type it is the literal's natural emission, so
/// a coercion site routed here is a pure superset of
/// [`Emit::emit`](crate::ast::Emit::emit).
pub trait Materialize<'context, 'block, 'state, 'scope> {
    /// The shared emission scope threaded into `materialize`.
    type Context;

    /// Emits this string literal as a value of `target_type`.
    fn materialize(
        &self,
        target_type: Type<'context>,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, crate::ast::Value<'context, 'block>>;
}
