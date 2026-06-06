//!
//! Contract storage layout: the slot assignment of state variables.
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;

use crate::ast::contract::ContractEmitter;

/// Storage location of a state variable in contract storage.
#[derive(Debug, Clone)]
pub struct StorageSlot {
    /// 256-bit storage slot index.
    pub slot: U256,
    /// Byte offset within the slot. Non-zero only for variables packed
    /// into a shared slot.
    pub byte_offset: u32,
    /// MLIR symbol name, formatted as `{label}_{node_id}` to match solc.
    /// The slang AST node id disambiguates like-named variables across
    /// inherited contracts.
    pub name: String,
}

impl StorageSlot {
    /// Creates a slot with `{label}_{node_id}` as the MLIR symbol name.
    pub fn new(slot: U256, byte_offset: u32, label: &str, node_id: impl std::fmt::Display) -> Self {
        Self {
            slot,
            byte_offset,
            name: format!("{label}_{node_id}"),
        }
    }
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Computes the storage layout using slang-solidity's ABI computation.
    ///
    /// Returns a mapping from state variable node ID to its storage slot
    /// (slot index and byte offset within the slot). Returns an empty map
    /// if the ABI is unavailable.
    pub fn compute_storage_layout(contract: &ContractDefinition) -> HashMap<NodeId, StorageSlot> {
        let Some(abi) = contract.compute_abi() else {
            return HashMap::new();
        };
        abi.storage_layout()
            .iter()
            .map(|item| {
                (
                    item.node_id(),
                    StorageSlot::new(
                        item.slot(),
                        item.offset() as u32,
                        item.label(),
                        item.node_id(),
                    ),
                )
            })
            .collect()
    }
}
