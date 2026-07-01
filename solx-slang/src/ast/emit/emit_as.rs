//!
//! Target-typed emission: emitting a node coerced to an expected MLIR type.
//!

use melior::ir::BlockRef;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a node coerced to an expected MLIR type: a node's argument or initialiser coercion.
///
/// The node emits its value, then casts to the target. `Target` is one `Type` for an expression.
pub trait EmitAs<'context: 'block, 'block, Target> {
    /// The coerced result: a single value.
    type Output;

    /// Emits this node coerced to `target`.
    fn emit_as<'state>(
        &self,
        target: Target,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Self::Output>;
}
