//!
//! The effect-position emission trait: an expression emitted for its side
//! effects, its value discarded.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::expression::ExpressionContext;

/// Emits an expression in statement position, such as `f();` or a for-loop step, for its side
/// effects, discarding the value. The value-less producers, a void call or `delete`, emit here.
pub trait EmitForEffect<'context: 'block, 'block> {
    /// Emits this expression for its side effects, returning the continuation.
    fn emit_for_effect<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;
}
