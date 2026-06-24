//!
//! How the `_;` placeholder of a modifier-wrapped function body is lowered.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Block;

use crate::ast::EmitStatement;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::modifier_parameter_binding::ModifierParameterBinding;
use crate::ast::contract::function::statement::StatementContext;

/// How the `_;` placeholder in a modifier-wrapped body is lowered.
///
/// A regular function's modifiers are separate `sol.func`s reached through a [`ModifierBodyCall`];
/// a constructor's run as an inline chain over the body statements.
#[derive(Default)]
pub enum ModifierStrategy<'context, 'block> {
    /// Not emitting a modifier-wrapped body: `_;` has no hand-off and emits
    /// nothing.
    #[default]
    None,
    /// A regular function: `_;` calls the wrapped body / next modifier-stage
    /// `sol.func`, threading the shared return values.
    BodyCall(ModifierBodyCall<'context, 'block>),
    /// A constructor's inline modifier chain: each stage is one modifier's body
    /// (the constructor body pushed as the final stage); a `_;` placeholder
    /// recurses to the next stage, emitting it through `Block::emit`.
    InlineChain {
        /// Each stage's body block (the constructor body is the last stage).
        stages: Vec<Block>,
        /// Each stage's parameter bindings, parallel to `stages`.
        parameters: Vec<Vec<ModifierParameterBinding<'context, 'block>>>,
        /// The stage currently being emitted.
        index: usize,
    },
}

impl<'context, 'block> ModifierStrategy<'context, 'block> {
    /// Emits the `_;` placeholder hand-off for the active strategy (also the inline chain's entry
    /// point). Taken by parameter rather than `&self` because the strategy lives inside `context`.
    pub fn emit_placeholder<'state>(
        context: &mut StatementContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        match &context.modifier_strategy {
            Self::InlineChain {
                stages,
                parameters,
                index,
            } => {
                let stage = *index;
                // A constructor has no return value, so the chain unwinds past the
                // last stage (no separate body call) — fall through, do not divert.
                let Some(stage_block) = stages.get(stage).cloned() else {
                    return Some(block);
                };
                let params = parameters.get(stage).cloned().unwrap_or_default();
                // Advance the cursor for the recursive `_;`, restore it after.
                if let Self::InlineChain { index, .. } = &mut context.modifier_strategy {
                    *index = stage + 1;
                }
                // The stage's parameters bracket the whole stage — including the
                // `_;` tail — in their own scope; the stage block opens its own.
                context.environment.enter_scope();
                for binding in params {
                    context
                        .environment
                        .define_variable(binding.declaration, binding.pointer);
                }
                let result = stage_block.emit(context, block);
                context.environment.exit_scope();
                if let Self::InlineChain { index, .. } = &mut context.modifier_strategy {
                    *index = stage;
                }
                result
            }
            Self::BodyCall(body_call) => {
                body_call.emit(&context.state.builder, &block);
                Some(block)
            }
            Self::None => Some(block),
        }
    }
}
