//!
//! State variable storage access: value-typed reads and writes.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use solx_utils::DataLocation;

use crate::ast::contract::function::storage_slot::StorageSlot;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a read of a value-typed state variable to `sol.addr_of` +
    /// `sol.load`.
    ///
    /// `constant` state variables and reference-typed slots (which yield a
    /// storage reference rather than a loaded value) are lowered by later
    /// domains.
    pub(super) fn emit_state_variable_read(
        &self,
        state_variable: &StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            unimplemented!("constant state variable read");
        }
        if declared_type.is_reference_type() {
            unimplemented!("reference-typed state variable read");
        }

        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .expect("every value-typed state variable has a storage slot");
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        let value = self.emit_storage_load(slot, element_type, &block)?;
        Ok((value, block))
    }

    /// Emits a value-typed storage load: `sol.addr_of` + `sol.load`.
    pub(super) fn emit_storage_load(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let pointer = self.emit_storage_addr_of(slot, element_type, block);
        self.state
            .builder
            .emit_sol_load(pointer, element_type, block)
    }

    /// Emits a value-typed storage store: `sol.addr_of` + `sol.store`.
    pub(super) fn emit_storage_store(
        &self,
        slot: &StorageSlot,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer = self.emit_storage_addr_of(slot, element_type, block);
        self.state.builder.emit_sol_store(value, pointer, block);
    }

    /// Returns a `!sol.ptr<element_type, Storage>` for a state variable slot.
    fn emit_storage_addr_of(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let pointer_type = self
            .state
            .builder
            .types
            .pointer(element_type, DataLocation::Storage);
        self.state
            .builder
            .emit_sol_addr_of(&slot.name, pointer_type, block)
    }
}
