//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
        &self,
        symbol_name: &str,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let pointer = self.emit_storage_addr_of(symbol_name, element_type, block);
        self.state
            .builder
            .emit_sol_load(pointer, element_type, block)
    }

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub fn emit_storage_store(
        &self,
        symbol_name: &str,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer = self.emit_storage_addr_of(symbol_name, element_type, block);
        self.state.builder.emit_sol_store(value, pointer, block);
    }

    /// Returns a `!sol.ptr<{element_type}, Storage>` pointer via `sol.addr_of`.
    fn emit_storage_addr_of(
        &self,
        symbol_name: &str,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let storage_pointer_type = self
            .state
            .builder
            .types
            .pointer(element_type, solx_utils::DataLocation::Storage);
        self.state
            .builder
            .emit_sol_addr_of(symbol_name, storage_pointer_type, block)
    }
}
