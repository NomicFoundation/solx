//!
//! Storage location of a state variable.
//!

use ruint::aliases::U256;

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
