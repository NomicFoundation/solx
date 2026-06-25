//!
//! The constructor-synthesis emission trait: a contract lowers its deploy-time construction into one
//! `sol.func` per constructor in the C3 chain, wired by `sol.call`.
//!

use std::collections::HashMap;

use melior::ir::BlockRef;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Environment;

use crate::ast::analysis::query::BaseConstructorArguments;
use crate::ast::contract::function::FunctionScope;

/// Synthesises a contract's deploy-time construction as a chain of `sol.func`s: the most-derived
/// `constructor()` (`kind = #Constructor`) and one plain internal `sol.func` per other constructor in
/// the linearisation, each `sol.call`ing the next, matching solc op-for-op.
pub trait EmitConstructor {
    /// Emits the contract's construction chain — the most-derived `constructor()` and a separate
    /// `sol.func` per other base constructor in the C3 linearisation, wired with `sol.call`.
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Emits one constructor as a `sol.func`: `owner`'s constructor (its parameters, mutability, body,
    /// and modifiers), chaining into the next constructor. The most-derived one (`is_most_derived`)
    /// additionally carries `kind = #Constructor` and runs the whole hierarchy's state-variable
    /// initializers; a base constructor is a plain internal func with a referenceable `id`.
    fn emit_constructor_func<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        is_most_derived: bool,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Emits the `sol.call` to the next constructor in the chain (if any): evaluates that base's
    /// invocation arguments in `owner`'s constructor scope and calls it, threading the block forward.
    fn emit_next_constructor_call<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        owner: &ContractDefinition,
        mro: &[ContractDefinition],
        base_arguments: &HashMap<NodeId, BaseConstructorArguments>,
        environment: &Environment<'context, 'block>,
        current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;

    /// Emits every state-variable inline initializer in the C3-linearised hierarchy, in order (a
    /// derived contract runs its bases' initializers and side effects, as solc does).
    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;
}
