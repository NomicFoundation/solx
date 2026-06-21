//!
//! The expression emission trait: each Slang expression node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::expression::ExpressionContext;

/// Emits a Slang expression node to MLIR, appending operations to `block`.
///
/// Implemented per node directly on the Slang AST type (the orphan rule forbids an
/// inherent method). The context is always the shared `&ExpressionContext` — an
/// expression declares no variables, so it never needs `&mut`. `Output` stays
/// associated because the expression family is not uniform: a value expression
/// yields a `BlockAnd<Value>`, a tuple-returning call or conditional and an
/// argument list a `BlockAnd<Vec<Value>>`.
///
/// `'context` (MLIR context) and `'block` (block region) are trait parameters
/// because `Output` names them — with `'context: 'block`, since a produced value
/// cannot outlive its block; `'state` (the emitter's field borrows) is a method
/// parameter, since it appears only in the threaded `&ExpressionContext` and never
/// in the result. Emission never fails — slang validates the source beforehand, so
/// an unsupported construct panics rather than returning an error.
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
