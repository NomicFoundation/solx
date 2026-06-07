//!
//! Contract storage layout: the slot assignment of state variables.
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use solx_utils::DataLocation;

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
    /// Storage class: `Storage` selects SLOAD/SSTORE, `Transient`
    /// (EIP-1153 `transient` variables) selects TLOAD/TSTORE. The two number
    /// their slots independently; the node-id-qualified symbol keeps them
    /// distinct without a name prefix.
    pub location: DataLocation,
}

impl StorageSlot {
    /// Creates a slot with `{label}_{node_id}` as the MLIR symbol name.
    pub fn new(
        slot: U256,
        byte_offset: u32,
        label: &str,
        node_id: impl std::fmt::Display,
        location: DataLocation,
    ) -> Self {
        Self {
            slot,
            byte_offset,
            name: format!("{label}_{node_id}"),
            location,
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
        // Persistent (`storage_layout`) and transient (`transient_storage_layout`)
        // variables both come from slang's ABI computation — never recomputed
        // here. The storage class is carried on the slot so each access selects
        // SLOAD/SSTORE versus TLOAD/TSTORE.
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
        // `immutable` variables have no native storage; slang lays them out as
        // storage slots appended after the persistent layout, and the recut
        // lowers them as ordinary storage so a read after the constructor's
        // write observes the assigned value.
        for item in abi.immutable_storage_layout() {
            layout.insert(
                item.node_id(),
                StorageSlot::new(
                    item.slot(),
                    item.offset() as u32,
                    item.label(),
                    item.node_id(),
                    DataLocation::Storage,
                ),
            );
        }
        layout
    }
}
