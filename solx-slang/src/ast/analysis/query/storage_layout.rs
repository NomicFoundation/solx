//!
//! Storage-layout query: re-keys Slang's ABI storage layout by node id (pure-Slang).
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;

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
    /// Storage class: `Storage` selects `SLOAD`/`SSTORE`, `Transient` (EIP-1153) selects
    /// `TLOAD`/`TSTORE`.
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

/// A contract's storage layout: state variable node ID to its storage slot.
pub trait StorageLayout {
    /// The layout re-keyed from Slang's ABI, mapping each state variable's node ID to its slot
    /// index, byte offset, and storage class (persistent, transient, or immutable). Empty when the
    /// ABI is unavailable.
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
                        solx_utils::DataLocation::Storage,
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
                    solx_utils::DataLocation::Transient,
                ),
            );
        }
        for member in self.members().iter() {
            let ContractMember::StateVariableDefinition(variable) = member else {
                continue;
            };
            if !matches!(variable.mutability(), StateVariableMutability::Immutable) {
                continue;
            }
            let node_id = variable.node_id();
            let label = variable.name().name();
            layout.insert(
                node_id,
                StorageSlot::new(
                    U256::ZERO,
                    0,
                    &label,
                    node_id,
                    solx_utils::DataLocation::Immutable,
                ),
            );
        }
        layout
    }
}
