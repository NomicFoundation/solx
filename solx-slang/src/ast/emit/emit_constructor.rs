//!
//! The constructor-synthesis emission trait: a contract emits its deploy-time construction as a
//! `sol.func`.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::FunctionEmitter;

/// Synthesises a contract's deploy-time construction as a `constructor()` `sol.func`: the declared
/// constructor when present, otherwise a synthesised one running the state-variable initializers.
pub trait EmitConstructor {
    /// Emits the contract's `constructor()` `sol.func`, threaded via the shared [`FunctionEmitter`].
    fn emit_constructor<'context>(
        &self,
        emitter: &FunctionEmitter<'_, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );
}
