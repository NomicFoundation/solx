//!
//! The function-definition emission trait: a function emits itself as a
//! `sol.func`.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::FunctionEmitter;

/// Emits a function definition as a `sol.func`. A contract threads the emission via the shared
/// [`FunctionEmitter`]; the node carries the definition.
pub trait EmitFunction {
    /// Opens the `sol.func`, binds parameters and return slots, threads the body statements, and
    /// closes with the default return, returning the emitted `sol.func` symbol name.
    ///
    /// `symbol_override` names a reached free function under its node-id-qualified symbol, suppressing
    /// the ABI dispatch entry a contract method would carry; a contract method passes `None`.
    fn emit<'context>(
        &self,
        emitter: &FunctionEmitter<'_, 'context>,
        symbol_override: Option<&str>,
        contract_body: &BlockRef<'context, '_>,
    ) -> String;
}
