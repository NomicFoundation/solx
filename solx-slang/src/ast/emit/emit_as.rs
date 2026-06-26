//!
//! Target-typed emission: emitting a node coerced to an expected MLIR type.
//!

use melior::ir::BlockRef;

use crate::ast::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a node coerced to an expected MLIR type (a node's argument / initialiser coercion).
///
/// Most expressions emit then cast to the target. The exception is a string literal, which toward
/// `bytesN` / `byte` materialises as a compile-time fixed-bytes constant rather than a runtime
/// `sol.string`. `Target` is one `Type` for an expression or a `&[Type]` signature for an argument list.
pub trait EmitAs<'context: 'block, 'block, Target> {
    /// The coerced result: a single value, or the coerced argument vector.
    type Output;

    /// Emits this node coerced to `target`.
    fn emit_as<'state>(
        &self,
        target: Target,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Self::Output>;
}
