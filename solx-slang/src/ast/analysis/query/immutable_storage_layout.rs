//!
//! Immutable layout query (solx-side).
//!
//! `immutable` state variables are emitted as `sol.immutable` (symbol-addressed, no storage slot),
//! read via `sol.load_immutable`, and written in the constructor through a `!sol.ptr<T, Immutable>`
//! store — matching solc. This query only ENUMERATES the immutables (keyed by node id, carrying each
//! variable's MLIR symbol name) so emission and reads can find them; the slot/offset it computes are
//! vestigial under the `Immutable` class and impose no storage layout.
//!

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::DataLocation;

use crate::ast::contract::storage_layout::StorageSlot;

/// A full 32-byte storage slot, matching Slang's `SemanticContext::SLOT_SIZE`.
const SLOT_SIZE: usize = 32;
/// An EVM address occupies 20 bytes, matching Slang's `ADDRESS_BYTE_SIZE`.
const ADDRESS_BYTE_SIZE: usize = 20;
/// A function selector occupies 4 bytes, matching Slang's `SELECTOR_SIZE`.
const SELECTOR_SIZE: usize = 4;

/// A contract's `immutable` storage layout, computed solx-side.
pub trait ImmutableStorageLayout {
    /// Lays out the `immutable` state variables as packed storage slots starting
    /// at `base_slot` (the persistent layout's high-water mark). Keyed by node id.
    fn immutable_storage_layout(&self, base_slot: U256) -> HashMap<NodeId, StorageSlot>;
}

impl ImmutableStorageLayout for ContractDefinition {
    fn immutable_storage_layout(&self, base_slot: U256) -> HashMap<NodeId, StorageSlot> {
        let mut layout = HashMap::new();
        let mut current_slot = base_slot;
        let mut byte_offset_in_slot: usize = 0;
        for variable in self.linearised_state_variables() {
            if !matches!(variable.mutability(), StateVariableMutability::Immutable) {
                continue;
            }
            let Some(variable_type) = variable.get_type() else {
                continue;
            };
            let Some(variable_size) = immutable_value_storage_size(&variable_type) else {
                continue;
            };

            // Pack into the current slot when it fits, otherwise start the next one.
            let remaining_bytes = SLOT_SIZE - byte_offset_in_slot;
            if byte_offset_in_slot > 0 && variable_size > remaining_bytes {
                current_slot += U256::from(1u64);
                byte_offset_in_slot = 0;
            }

            let label = variable.name().unparse().to_string();
            let node_id = variable.node_id();
            // `immutable`s are emitted as `sol.immutable` (a symbol, not a storage slot) and read via
            // `sol.load_immutable`, matching solc. The slot/offset are vestigial under the `Immutable`
            // class — kept only so the entry carries the variable's MLIR symbol name.
            layout.insert(
                node_id,
                StorageSlot::new(
                    current_slot,
                    byte_offset_in_slot as u32,
                    &label,
                    node_id,
                    DataLocation::Immutable,
                ),
            );

            byte_offset_in_slot += variable_size;
            current_slot += U256::from(byte_offset_in_slot / SLOT_SIZE);
            byte_offset_in_slot %= SLOT_SIZE;
        }
        layout
    }
}

/// Storage size in bytes of a value type that an `immutable` can hold, mirroring
/// Slang's `storage_size_of_type_id` for the value-type arms (immutables are
/// restricted to value types). Returns `None` for anything unexpected.
fn immutable_value_storage_size(slang_type: &SlangType) -> Option<usize> {
    Some(match slang_type {
        SlangType::Integer(integer_type) => (integer_type.bits() as usize).div_ceil(8),
        SlangType::Boolean(_) => 1,
        SlangType::Address(_) | SlangType::Contract(_) | SlangType::Interface(_) => {
            ADDRESS_BYTE_SIZE
        }
        SlangType::ByteArray(byte_array_type) => byte_array_type.width() as usize,
        SlangType::Enum(_) => 1,
        // An external function ref is address + selector; an internal one is an
        // opaque 8-byte value (matching Slang's `storage_size_of_type_id`).
        SlangType::Function(function_type) => {
            if function_type.is_externally_visible() {
                ADDRESS_BYTE_SIZE + SELECTOR_SIZE
            } else {
                8
            }
        }
        SlangType::UserDefinedValue(udvt) => {
            return immutable_value_storage_size(&udvt.target_type()?);
        }
        _ => return None,
    })
}
