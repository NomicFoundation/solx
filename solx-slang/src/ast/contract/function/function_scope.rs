//!
//! The scope threaded through a function's and constructor's emission.
//!

use std::collections::HashMap;

use melior::ir::Type;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;

use crate::ast::analysis::query::modifier_resolution::ModifierResolution;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::storage_layout::StorageSlot;

/// The pure-data scope threaded through function and constructor emission.
pub struct FunctionScope<'state, 'context> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Containing contract; `None` for a library's functions, which have no constructor or state.
    pub contract: Option<&'state ContractDefinition>,
    /// MLIR type of the contract or library being emitted, the type of `this`.
    pub contract_type: Option<Type<'context>>,
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
        contract_type: Option<Type<'context>>,
        dispatch: &'state ContractDispatch,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            contract,
            contract_type,
            dispatch,
            storage_layout,
        }
    }

    /// Resolves a modifier invocation to the body-bearing definition to emit, or `None` to skip a
    /// non-modifier invocation or a bodyless modifier. Applies lexical resolution, then virtual
    /// override re-dispatch.
    pub fn resolve_modifier_invocation(
        &self,
        invocation: &ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let Some(Definition::Modifier(lexical)) = invocation.name().resolve_to_definition() else {
            return None;
        };
        let definition = self
            .contract
            .and_then(|contract| contract.resolve_modifier_override(invocation, &lexical))
            .unwrap_or(lexical);
        definition.body().is_some().then_some(definition)
    }
}
