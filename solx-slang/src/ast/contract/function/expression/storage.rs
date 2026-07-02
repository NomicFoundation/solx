//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let pointer = self.emit_storage_addr_of(slot, element_type, block);
        Pointer::new(pointer)
            .load(AstType::new(element_type), self.state, block)
            .into_mlir()
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
        Pointer::new(pointer).store(AstValue::new(value), self.state, block);
    }

    /// Emits every state-variable inline initializer (`T x = <expr>;`)
    /// declared in `contract`, in source order.
    ///
    /// Reference-typed slots are written via `sol.copy` from the evaluated
    /// value into the storage reference. Value-typed slots cast to the
    /// declared element type and store via `sol.store`.
    pub fn emit_state_var_initializers(
        &self,
        contract: &ContractDefinition,
        mut block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
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
            let declared_type = state_variable
                .get_type()
                .expect("binder types every state variable");
            let element_type =
                TypeConversion::resolve_slang_type(&declared_type, None, self.state);
            let address_type = Self::address_type(
                self.state,
                element_type,
                slot.location,
                &declared_type,
            );
            let storage_ref =
                Pointer::addr_of(&slot.name, AstType::new(address_type), self.state, &block);
            let BlockAnd {
                value,
                block: next_block,
            } = initializer.emit(self, block);
            block = next_block;
            if declared_type.is_reference_type() {
                storage_ref.copy_from(AstValue::new(value), self.state, &block);
            } else {
                let stored_value = TypeConversion::from_target_type(element_type, self.state)
                    .emit(value, self.state, &block);
                storage_ref.store(AstValue::new(stored_value), self.state, &block);
            }
        }
        block
    }

    /// Returns a `!sol.ptr<element_type, location>` pointer via `sol.addr_of`, at the slot's storage
    /// class: persistent `Storage` or `Transient`.
    fn emit_storage_addr_of(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let pointer_type =
            AstType::pointer(self.state.mlir_context, element_type, slot.location)
                .into_mlir();
        Pointer::addr_of(&slot.name, AstType::new(pointer_type), self.state, block).into_mlir()
    }
}
