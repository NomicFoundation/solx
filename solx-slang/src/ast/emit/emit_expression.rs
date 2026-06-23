//!
//! The expression emission trait: each Slang expression node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a Slang expression node to MLIR, appending operations to `block`.
///
/// Implemented per node directly on the Slang AST type (the orphan rule forbids an inherent method).
/// The context is the shared `&ExpressionContext`; `Output` is associated because the family is not
/// uniform. Emission never fails — slang validated the source, so an unsupported construct panics.
pub trait EmitExpression<'context: 'block, 'block> {
    /// The node family's result: one value, a value list, or nothing.
    type Output;

    /// Emits this expression into `block`.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output;
}
