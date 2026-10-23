//!
//! The storage slot record of one state variable.
//!

use ruint::aliases::U256;
use slang_solidity_v2::abi::StorageVariable;

/// Storage location of a state variable in contract storage.
#[derive(Debug, Clone)]
pub struct StorageSlot {
    /// 256-bit storage slot index.
    pub slot: U256,
    /// Byte offset within the slot. Non-zero only for variables packed
    /// into a shared slot.
    pub byte_offset: u32,
}

impl From<&StorageVariable> for StorageSlot {
    fn from(variable: &StorageVariable) -> Self {
        Self {
            slot: variable.slot(),
            byte_offset: variable.offset() as u32,
        }
    }
}
