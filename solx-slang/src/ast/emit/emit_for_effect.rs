//!
//! The effect-position emission trait: an expression emitted for its side
//! effects, its value discarded.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::expression::ExpressionContext;

/// Emits an expression in statement position — an expression statement (`f();`)
/// or a for-loop step (`i++`) — for its side effects, discarding the value and
/// yielding only the continuation block.
///
/// The two value-less producers, a void call and `delete`, never reach value
/// position ([`EmitExpression::emit`](crate::ast::EmitExpression) always yields a
/// [`Value`](crate::ast::Value)), so they lower here rather than through value
/// emission. The context is the shared `&ExpressionContext`, as for every
/// expression-family mode.
pub trait EmitForEffect<'context: 'block, 'block> {
    /// Emits this expression for its side effects, returning the continuation.
    fn emit_for_effect<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;
}
