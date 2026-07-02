//!
//! Contract storage layout: the slot assignment of state variables.
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::NodeId;

/// Storage location of a state variable in contract storage.
#[derive(Debug, Clone)]
pub struct StorageSlot {
    /// 256-bit storage slot index.
    pub slot: U256,
    /// Byte offset within the slot. Non-zero only for variables packed
    /// into a shared slot.
    pub byte_offset: u32,
    /// MLIR symbol name, formatted as `{label}_{node_id}`.
    pub name: String,
    /// Storage class: `Storage` selects SLOAD/SSTORE, `Transient` (EIP-1153) selects TLOAD/TSTORE.
    pub location: solx_utils::DataLocation,
}

impl StorageSlot {
    /// Creates a slot with `{label}_{node_id}` as the MLIR symbol name.
    pub fn new(
        slot: U256,
        byte_offset: u32,
        label: &str,
        node_id: impl std::fmt::Display,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self {
            slot,
            byte_offset,
            name: format!("{label}_{node_id}"),
            location,
        }
    }
}

/// The panic-on-missing lookup of a state variable's storage slot, shared by the emitters that read
/// or write storage.
pub trait StateVariableSlot {
    /// The slot registered for `node_id`. Slang registers every state variable it linearises into the
    /// storage layout, so a missing entry is a frontend invariant violation, not a runtime condition.
    fn slot(&self, node_id: NodeId) -> &StorageSlot;
}

impl StateVariableSlot for HashMap<NodeId, StorageSlot> {
    fn slot(&self, node_id: NodeId) -> &StorageSlot {
        self.get(&node_id)
            .unwrap_or_else(|| unreachable!("unregistered state variable {node_id:?}"))
    }
}
