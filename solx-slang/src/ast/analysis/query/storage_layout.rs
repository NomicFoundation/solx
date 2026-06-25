//!
//! Storage-layout query: re-keys Slang's ABI storage layout by node id (pure-Slang).
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use solx_utils::DataLocation;

use crate::ast::contract::storage_layout::StorageSlot;

use super::immutable_storage_layout::ImmutableStorageLayout;

/// A contract's storage layout: state variable node ID → its storage slot.
pub trait StorageLayout {
    /// The layout re-keyed from Slang's ABI (persistent, transient, and immutable
    /// layouts), never re-packed here. Empty when the ABI is unavailable.
    fn storage_layout(&self) -> HashMap<NodeId, StorageSlot>;
}

impl StorageLayout for ContractDefinition {
    fn storage_layout(&self) -> HashMap<NodeId, StorageSlot> {
        let Some(abi) = self.compute_abi() else {
            return HashMap::new();
        };
        let mut layout: HashMap<NodeId, StorageSlot> = abi
            .storage_layout()
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
            .collect();
        // Transient (EIP-1153) variables number their slots in a separate space;
        // the storage class on the slot selects TLOAD/TSTORE over SLOAD/SSTORE.
        for item in abi.transient_storage_layout() {
            layout.insert(
                item.node_id(),
                StorageSlot::new(
                    item.slot(),
                    item.offset() as u32,
                    item.label(),
                    item.node_id(),
                    DataLocation::Transient,
                ),
            );
        }
        // `immutable` variables carry no storage slot: they are emitted as `sol.immutable` and read
        // via `sol.load_immutable`. This only adds an `Immutable`-located entry per immutable so
        // emission and reads can resolve it by node id to its MLIR symbol name.
        for (node_id, slot) in self.immutable_storage_layout() {
            layout.insert(node_id, slot);
        }
        layout
    }
}
