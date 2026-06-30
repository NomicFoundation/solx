//!
//! The scope threaded through a function's and constructor's emission.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;

use crate::ast::analysis::query::ModifierResolution;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::storage_layout::StorageSlot;

/// The pure-data scope threaded through function and constructor emission.
pub struct FunctionScope<'state, 'context> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Containing contract; `None` for a library's functions, which have no constructor or state.
    pub contract: Option<&'state ContractDefinition>,
    /// Contract-local dispatch metadata.
    pub dispatch: &'state ContractDispatch,
    /// State variable node ID to `(slot, byte_offset)` mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
}

impl<'state, 'context> FunctionScope<'state, 'context> {
    /// Bundles the references function emission threads in common.
    pub fn new(
        state: &'state Context<'context>,
        contract: Option<&'state ContractDefinition>,
        dispatch: &'state ContractDispatch,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            contract,
            dispatch,
            storage_layout,
        }
    }

    /// Resolves a modifier invocation to the body-bearing definition to emit, or `None` to skip it
    /// (unresolvable, or a modifier with no body). Applies lexical resolution, qualified-name
    /// fallback, then virtual override re-dispatch.
    pub fn resolve_modifier_invocation(
        &self,
        invocation: &ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let lexical = match invocation.name().resolve_to_definition() {
            Some(Definition::Modifier(modifier)) => modifier,
            _ => self
                .contract
                .and_then(|contract| contract.resolve_qualified_modifier(invocation))?,
        };
        let definition = self
            .contract
            .and_then(|contract| contract.resolve_modifier_override(invocation, &lexical))
            .unwrap_or(lexical);
        definition.body().is_some().then_some(definition)
    }
}
