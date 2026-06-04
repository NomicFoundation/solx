//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
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

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub fn emit_storage_store(
        &self,
        slot: &StorageSlot,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer = self.emit_storage_addr_of(slot, element_type, block);
        self.state.builder.emit_sol_store(value, pointer, block);
    }

    /// Emits every state-variable inline initializer (`T x = <expr>;`)
    /// declared in `contract`, in source order.
    ///
    /// Reference-typed slots are written via `sol.copy` from the evaluated
    /// value into the storage reference. Value-typed slots cast to the
    /// declared element type and store via `sol.store`.
    ///
    /// # Errors
    ///
    /// Returns an error if any initializer expression has an unresolved type
    /// or contains unsupported constructs.
    pub fn emit_state_var_initializers(
        &self,
        contract: &ContractDefinition,
        mut block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        for member in contract.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(slot) = self.storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            if matches!(initializer, Expression::ArrayExpression(_)) {
                unimplemented!("array-literal state variable initializers are not yet supported");
            }
            let declared_type = state_variable
                .get_type()
                .expect("the binder types every state variable");
            let builder = &self.state.builder;
            let element_type = TypeConversion::resolve_slang_type(&declared_type, None, builder);
            let address_type =
                Self::address_type(builder, element_type, DataLocation::Storage, &declared_type);
            let storage_ref = builder.emit_sol_addr_of(&slot.name, address_type, &block);
            let (value, next_block) = self.emit_value(&initializer, block)?;
            block = next_block;
            if declared_type.is_reference_type() {
                builder.emit_sol_copy(value, storage_ref, &block);
            } else {
                let stored_value = TypeConversion::from_target_type(element_type, builder)
                    .emit(value, builder, &block);
                builder.emit_sol_store(stored_value, storage_ref, &block);
            }
        }
        Ok(block)
    }

    /// Returns a `!sol.ptr<element_type, Storage>` pointer via `sol.addr_of`.
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
