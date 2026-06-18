//!
//! The lvalue emission trait: an assignable expression emits its [`Place`].
//!

use melior::ir::BlockRef;

use crate::ast::BlockAnd;
use crate::ast::Place;
use crate::ast::contract::function::expression::ExpressionContext;

/// Emits the [`Place`] an assignable expression denotes (its `!sol.ptr` and
/// element type), without the trailing `sol.load` / `sol.store`. A readable
/// place's value emission loads from it and an assignment stores into it, so the
/// address is computed once here and shared by both. The context is the shared
/// `&ExpressionContext` expression emission threads — an lvalue declares no
/// variables.
pub trait EmitPlace<'context: 'block, 'block> {
    /// Emits this expression's place into `block`.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>>;
}
