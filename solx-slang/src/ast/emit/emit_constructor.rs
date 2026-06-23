//!
//! The constructor-synthesis emission trait: a contract lowers its deploy-time
//! `constructor()`.
//!

use melior::ir::BlockRef;

use crate::ast::contract::function::FunctionScope;

/// Synthesises a contract's deploy-time constructor as a `sol.func` (possibly emitting a `sol.func`
/// with no source `FunctionDefinition`).
pub trait EmitConstructor {
    /// Emits the contract's constructor as a `sol.func` — the contract's own
    /// constructor when it declares one, otherwise an empty `constructor()`
    /// running just the state-variable initializers.
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Emits every state-variable inline initializer (`T x = <expr>;`) declared
    /// in the contract, in source order.
    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;
}
