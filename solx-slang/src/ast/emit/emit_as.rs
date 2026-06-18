//!
//! Target-typed emission: emitting a node coerced to an expected MLIR type.
//!

use melior::ir::BlockRef;

use crate::ast::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a node coerced to an expected MLIR type — the projection's argument /
/// initialiser coercion, a superset of [`EmitExpression`](crate::ast::EmitExpression).
///
/// Most expressions emit naturally and then cast to the target. The exception is
/// a string literal: slang types it `Literal(String)` context-free, so toward
/// `bytesN` / `byte` it must materialise as a compile-time, left-aligned
/// fixed-bytes / byte constant — the literal in the high bytes, zero-padded right
/// — rather than a runtime `sol.string` the integer-only verifier rejects; the
/// target reaches the literal only from the use site. `Target` is a single
/// [`Type`](melior::ir::Type) for one expression and a `&[Type]` signature for an
/// ordered argument list, each element coerced to its parameter type — hence the
/// generic `Target` and associated `Output`.
pub trait EmitAs<'context: 'block, 'block, Target> {
    /// The coerced result — a single value, or the coerced argument vector.
    type Output;

    /// Emits this node coerced to `target`.
    fn emit_as<'state>(
        &self,
        target: Target,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Self::Output>;
}
