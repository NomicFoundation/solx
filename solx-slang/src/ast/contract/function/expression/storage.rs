//!
//! Storage load/store expression lowering via `inttoptr`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use solx_utils::AddressSpace;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.load`).
    pub fn emit_storage_load(
        &self,
        // TODO: change to i256
        slot: u64,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let i256 = self.state.i256();
        let storage_pointer_type = self.state.pointer(AddressSpace::Storage);
        let slot_value = self.state.builder().emit_i256_from_u64(slot, block)?;
        let pointer = self
            .state
            .builder()
            .emit_inttoptr(slot_value, storage_pointer_type, block);
        self.state.builder().emit_load(pointer, i256, block)
    }

    /// Emits a storage store (`inttoptr` slot to `ptr addrspace(5)`, then `llvm.store`).
    ///
    /// # Errors
    ///
    /// Returns an error if the slot constant cannot be emitted.
    pub fn emit_storage_store(
        &self,
        // TODO: change to i256
        slot: u64,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let storage_pointer_type = self.state.pointer(AddressSpace::Storage);
        let slot_value = self.state.builder().emit_i256_from_u64(slot, block)?;
        let pointer = self
            .state
            .builder()
            .emit_inttoptr(slot_value, storage_pointer_type, block);
        self.state.builder().emit_store(value, pointer, block);
        Ok(())
    }
}
