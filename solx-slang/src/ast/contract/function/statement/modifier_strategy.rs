//!
//! How the `_;` placeholder statement is lowered while emitting a modifier definition's body.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;

use solx_mlir::ods::sol::PlaceholderOperation;

use crate::ast::contract::function::statement::StatementContext;

/// How a `_;` placeholder statement is lowered.
///
/// Only a `sol.modifier` definition body contains `_;`, lowered to `sol.placeholder`. Every other
/// body ([`Self::None`]) carries no placeholder, so the strategy is a no-op there.
#[derive(Default)]
pub enum ModifierStrategy {
    /// Not a modifier-definition body: a `_;` placeholder cannot occur, so it emits nothing.
    #[default]
    None,
    /// A `sol.modifier` definition body: `_;` lowers to `sol.placeholder`.
    Placeholder,
}

impl ModifierStrategy {
    /// Emits the `_;` placeholder per the active strategy. Taken by parameter rather than `&self`
    /// because the strategy lives inside `context`.
    pub fn emit_placeholder<'state, 'context, 'block>(
        context: &mut StatementContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        match context.modifier_strategy {
            Self::Placeholder => {
                block.append_operation(mlir_op_build!(
                    &context.state.builder,
                    PlaceholderOperation
                ));
                Some(block)
            }
            Self::None => Some(block),
        }
    }
}
