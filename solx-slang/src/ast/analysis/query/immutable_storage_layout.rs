//!
//! Immutable layout query (solx-side).
//!
//! `immutable` state variables are emitted as `sol.immutable` (symbol-addressed, no storage slot),
//! read via `sol.load_immutable`, and written in the constructor through a `!sol.ptr<T, Immutable>`
//! store — matching solc. This query only ENUMERATES the immutables (keyed by node id, carrying each
//! variable's MLIR symbol name) so emission and reads can find them; an `immutable` occupies no
//! storage slot, so no slot/offset is computed.
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use solx_utils::DataLocation;

use crate::ast::contract::storage_layout::StorageSlot;

/// A contract's `immutable` layout, computed solx-side.
pub trait ImmutableStorageLayout {
    /// Enumerates the `immutable` state variables as `Immutable`-located entries keyed by node id,
    /// each carrying the variable's MLIR symbol name. An `immutable` occupies no storage slot, so the
    /// entry's slot/offset are unused placeholders.
    fn immutable_storage_layout(&self) -> HashMap<NodeId, StorageSlot>;
}

impl ImmutableStorageLayout for ContractDefinition {
    fn immutable_storage_layout(&self) -> HashMap<NodeId, StorageSlot> {
        let mut layout = HashMap::new();
        for variable in self.linearised_state_variables() {
            if !matches!(variable.mutability(), StateVariableMutability::Immutable) {
                continue;
            }
            let label = variable.name().unparse().to_string();
            let node_id = variable.node_id();
            // An `immutable` is emitted as `sol.immutable` (a symbol, not a storage slot) and read via
            // `sol.load_immutable`, matching solc — so the slot/offset are unused placeholders, kept
            // only because the `StorageSlot` entry carries the variable's MLIR symbol name.
            layout.insert(
                node_id,
                StorageSlot::new(U256::ZERO, 0, &label, node_id, DataLocation::Immutable),
            );
        }
        layout
    }
}
