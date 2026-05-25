//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;

use ruint::aliases::U256;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
        &self,
        slot: U256,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let pointer = self.emit_storage_addr_of(slot, element_type, block);
        self.state
            .builder
            .emit_sol_load(pointer, element_type, block)
    }

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub fn emit_storage_store(
        &self,
        slot: U256,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer = self.emit_storage_addr_of(slot, value.r#type(), block);
        self.state.builder.emit_sol_store(value, pointer, block);
    }

    /// Returns a `!sol.ptr<element_type, Storage>` pointer via `sol.addr_of`.
    fn emit_storage_addr_of(
        &self,
        slot: U256,
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
            .emit_sol_addr_of(&format!("slot_{slot}"), pointer_type, block)
    }
}
