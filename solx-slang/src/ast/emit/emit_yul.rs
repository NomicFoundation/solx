//!
//! The inline-assembly (Yul) emission trait: each Yul node emits its own MLIR.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::statement::assembly::YulContext;

/// Emits an inline-assembly (Yul) node to MLIR, appending operations to `block`.
///
/// Implemented per node directly on the Slang AST type, since the orphan rule forbids an inherent method.
/// The context is `&mut YulContext`, since a Yul `let` declares variables; `Output` is associated because
/// the family is not uniform. Emission never fails: slang validated the source.
pub trait EmitYul<'context: 'block, 'block> {
    /// The node's result: a continuation, or a value (or values) paired with one.
    type Output;

    /// Emits this Yul node into `block`.
    fn emit<'state>(
        &self,
        context: &mut YulContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output;
}
