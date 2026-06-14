//!
//! How the `_;` placeholder of a modifier-wrapped function body is lowered.
//!

use slang_solidity_v2::ast::Statements;

use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::modifier_parameter_binding::ModifierParameterBinding;

/// How the `_;` placeholder in a modifier-wrapped body is lowered.
///
/// A regular function's modifiers are emitted as separate `sol.func`s reached
/// through a [`ModifierBodyCall`] hand-off; a constructor's modifiers run as an
/// inline chain over the body statements. Replaces four parallel fields
/// (`modifier_body_call`, `modifier_stages`, `modifier_stage_params`,
/// `modifier_stage_index`) whose combination encoded these mutually exclusive
/// modes.
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
    /// statements (the constructor body pushed as the final stage); a `_;`
    /// placeholder recurses to the next stage.
    InlineChain {
        /// Each stage's body statements (the constructor body is the last stage).
        stages: Vec<Statements>,
        /// Each stage's parameter bindings, parallel to `stages`.
        parameters: Vec<Vec<ModifierParameterBinding<'context, 'block>>>,
        /// The stage currently being emitted.
        index: usize,
    },
}
