//!
//! Storage-layout query: re-keys Slang's ABI storage layout by node id (pure-Slang).
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use solx_utils::DataLocation;

use crate::ast::contract::storage_layout::StorageSlot;

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
        // via `sol.load_immutable`. Slang's ABI layout omits them, so enumerate them from the AST —
        // one `Immutable`-located entry per immutable, carrying the variable's MLIR symbol name so
        // emission and reads resolve it by node id (the slot/offset are unused placeholders).
        for variable in self.linearised_state_variables() {
            if !matches!(variable.mutability(), StateVariableMutability::Immutable) {
                continue;
            }
            let node_id = variable.node_id();
            let label = variable.name().unparse().to_string();
            layout.insert(
                node_id,
                StorageSlot::new(U256::ZERO, 0, &label, node_id, DataLocation::Immutable),
            );
        }
        layout
    }
}
