//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
        &self,
        slot: u64,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let pointer = self.emit_storage_addr_of(slot, block);
        let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
        self.state.builder.emit_sol_load(pointer, ui256, block)
    }

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub fn emit_storage_store(
        &self,
        slot: u64,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer = self.emit_storage_addr_of(slot, block);
        self.state.builder.emit_sol_store(value, pointer, block);
    }

    /// Returns a `!sol.ptr<ui256, Storage>` pointer via `sol.addr_of`.
    fn emit_storage_addr_of(
        &self,
        slot: u64,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let storage_pointer_type = self
            .state
            .builder
            .get_type(solx_mlir::Builder::SOL_PTR_STORAGE);
        self.state
            .builder
            .emit_sol_addr_of(&format!("slot_{slot}"), storage_pointer_type, block)
    }
}
