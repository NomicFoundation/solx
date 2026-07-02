//!
//! The function-definition emission trait: a function emits itself as a
//! `sol.func`.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::function_scope::FunctionScope;

/// Emits a function definition as a `sol.func`. A contract or library threads the
/// emission via the shared [`FunctionScope`]; the node carries the projection.
pub trait EmitFunction {
    /// Opens the `sol.func`, binds parameters and return slots, emits any modifier
    /// `sol.modifier_call_blk`s, threads the body statements, and closes with the default return.
    /// `symbol_override` names the `sol.func` explicitly and suppresses the public selector and
    /// special kind, for free and shadowed-base functions that are never dispatched; `None` uses
    /// the function's canonical, dispatchable symbol.
    fn emit<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    );
}
