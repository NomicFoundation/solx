//!
//! The constructor-synthesis emission trait: a contract lowers its deploy-time
//! `constructor()` from the C3 inheritance chain.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::BlockRef;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Environment;

use crate::ast::contract::function::FunctionScope;

/// Synthesises a contract's deploy-time constructor as a `sol.func`, walking the C3 chain (and
/// possibly emitting a `sol.func` with no source `FunctionDefinition`).
pub trait EmitConstructor {
    /// Emits the contract's constructor as a `sol.func` — the contract's own
    /// constructor when no base contributes one, an empty `constructor()` running
    /// just the state-variable initializers, or the inheritance-chain construction.
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    );

    /// Emits every state-variable inline initializer in the C3-linearised hierarchy, in order (a
    /// derived contract runs its bases' initializers and side effects, as solc does).
    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;

    /// Binds each base constructor's parameters into its own scope, in C3 order,
    /// threading the entry block forward (argument evaluation has side effects).
    fn bind_base_constructor_scopes<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &mut HashSet<NodeId>,
        current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block>;

    /// Emits each base constructor's body, base-first (reversed MRO), each in its
    /// own parameter scope, closing the constructor with a `sol.return` unless a
    /// body already terminated the block.
    fn emit_constructor_bodies<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mro: &[ContractDefinition],
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &HashSet<NodeId>,
        entry: &BlockRef<'context, 'block>,
        current_block: BlockRef<'context, 'block>,
    );
}
