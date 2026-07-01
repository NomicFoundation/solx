//!
//! The lvalue emission trait: an assignable expression emits its [`Place`].
//!

use melior::ir::BlockRef;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::place::Place;

/// Emits the [`Place`] an assignable expression denotes, its `!sol.ptr` and element type, without
/// the load or store, so a read and an assignment share one address computation.
pub trait EmitPlace<'context: 'block, 'block> {
    /// Emits this expression's place into `block`.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>>;
}
