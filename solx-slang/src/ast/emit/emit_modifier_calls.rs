//!
//! The modifier-call emission trait: a modified function emits one `sol.modifier_call_blk` per
//! invocation at the top of its body, and its invoked modifiers are emitted once as `sol.modifier` defs.
//!

use melior::ir::BlockRef;
use melior::ir::Type;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Parameter;

use crate::ast::contract::function::FunctionScope;

/// Emits a function's (or constructor's) modifier invocations as `sol.modifier_call_blk` ops, and
/// resolves the invoked modifier definitions so the contract can emit their `sol.modifier` defs.
pub trait EmitModifierCalls {
    /// The override-resolved, body-bearing modifier definitions this function invokes, in source
    /// order, with base-constructor invocations excluded. The contract dedups these into `sol.modifier` defs.
    fn resolve_invoked_modifiers<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
    ) -> Vec<FunctionDefinition>;

    /// Emits one `sol.modifier_call_blk` per modifier invocation, appended to `function_block` before
    /// the wrapped function's inlined body. Each block carries a fresh copy of `parameters` as block
    /// arguments, evaluates the invocation arguments against them, and `sol.call`s the modifier.
    fn emit_modifier_call_blocks<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        parameters: &[Parameter],
        parameter_types: &[Type<'context>],
        function_block: &BlockRef<'context, 'block>,
    );

    /// Emits this modifier definition as a contract-level `sol.modifier`: its parameters as block
    /// arguments spilled to stack slots, its statements with `_;` emitted as `sol.placeholder`, and a
    /// terminating `sol.return`.
    fn emit_modifier_definition<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );
}
