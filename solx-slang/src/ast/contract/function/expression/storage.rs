//!
//! State variable storage access: reads, writes, and inline initializers.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits the contract's inline state-variable initializers (`T x = expr;`)
    /// in declaration order, returning the continuation block.
    ///
    /// A value-typed slot casts its initializer and stores it; a reference-typed
    /// slot (`string s = "…";`) copies the evaluated reference into the storage
    /// reference with `sol.copy`. A slot without an initializer is skipped.
    /// Array-literal initializers defer to a later domain.
    pub fn emit_state_var_initializers(
        &self,
        contract: &ContractDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let mut block = block;
        for member in contract.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            if matches!(initializer, Expression::ArrayExpression(_)) {
                unimplemented!("array-literal state variable initializer");
            }
            let declared_type = state_variable
                .get_type()
                .expect("the binder types every state variable");
            let element_type =
                TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
            let slot = self
                .storage_layout
                .get(&state_variable.node_id())
                .expect("every state variable has a storage slot");

            let storage_reference = if declared_type.is_reference_type() {
                self.state
                    .builder
                    .emit_sol_addr_of(&slot.name, element_type, &block)
            } else {
                self.emit_storage_addr_of(slot, element_type, &block)
            };
            let (value, next_block) = self.emit_value(&initializer, block)?;
            block = next_block;
            if declared_type.is_reference_type() {
                self.state
                    .builder
                    .emit_sol_copy(value, storage_reference, &block);
            } else {
                let value = TypeConversion::from_target_type(element_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                self.state
                    .builder
                    .emit_sol_store(value, storage_reference, &block);
            }
        }
        Ok(block)
    }

    /// Lowers a read of a state variable.
    ///
    /// A value-typed slot loads its scalar via `sol.addr_of` + `sol.load`; a
    /// reference-typed slot (struct / array / mapping / `bytes` in storage)
    /// yields the storage reference itself via `sol.addr_of`, addressed in
    /// place with no scalar to load. `constant` state variables are lowered by
    /// a later domain.
    pub fn emit_state_variable_read(
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

        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .expect("every state variable has a storage slot");
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        if declared_type.is_reference_type() {
            let reference = self
                .state
                .builder
                .emit_sol_addr_of(&slot.name, element_type, &block);
            return Ok((reference, block));
        }
        let value = self.emit_storage_load(slot, element_type, &block)?;
        Ok((value, block))
    }

    /// Emits a value-typed storage load: `sol.addr_of` + `sol.load`.
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

    /// Emits a value-typed storage store: `sol.addr_of` + `sol.store`.
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
