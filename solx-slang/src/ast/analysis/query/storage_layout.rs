//!
//! Storage-layout query: re-keys Slang's ABI storage layout by node id (pure-Slang).
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use solx_utils::DataLocation;

use crate::ast::contract::storage_layout::StorageSlot;

/// A contract's storage layout: state variable node ID → its storage slot.
pub trait StorageLayout {
    /// The layout re-keyed from Slang's ABI persistent storage layout, never
    /// re-packed here. Empty when the ABI is unavailable.
    fn storage_layout(&self) -> HashMap<NodeId, StorageSlot>;
}

impl StorageLayout for ContractDefinition {
    fn storage_layout(&self) -> HashMap<NodeId, StorageSlot> {
        let Some(abi) = self.compute_abi() else {
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
                        DataLocation::Storage,
                    ),
                )
            })
            .collect()
    }
}
