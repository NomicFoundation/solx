//!
//! The scope threaded through a function's (and constructor's) emission.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;

use crate::ast::contract::storage_layout::StorageSlot;

/// The pure-data scope threaded through function and constructor emission.
pub struct FunctionScope<'state, 'context> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Containing contract (`None` for a library's functions, which have no constructor / state).
    pub contract: Option<&'state ContractDefinition>,
    /// State variable node ID to `(slot, byte_offset)` mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
}

impl<'state, 'context> FunctionScope<'state, 'context> {
    /// Bundles the references function emission threads in common.
    pub fn new(
        state: &'state Context<'context>,
        contract: Option<&'state ContractDefinition>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            contract,
            storage_layout,
        }
    }
}
