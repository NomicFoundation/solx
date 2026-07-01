//!
//! The multi-value emission trait: an expression that produces several values emits its value list.
//!

use melior::ir::BlockRef;
use melior::ir::Value;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a multi-valued expression to all its values: a tuple yields its elements, a call or a
/// conditional its results. Used where several values are consumed at once: a tuple return, a
/// destructuring assignment or declaration, a tuple-typed conditional branch.
pub trait EmitValues<'context: 'block, 'block> {
    /// Emits this expression's values into `block`.
    fn emit_values<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;
}
