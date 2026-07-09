//!
//! Storage load/store expression lowering via Sol dialect.
//!

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

impl<'state, 'context> ExpressionEmitter<'state, 'context> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub fn emit_storage_load(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        context: &Context<'context>,
    ) -> Value<'context> {
        let pointer = self.emit_storage_addr_of(slot, element_type, context);
        pointer.load(element_type, context)
    }

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub fn emit_storage_store(
        &self,
        slot: &StorageSlot,
        value: Value<'context>,
        element_type: Type<'context>,
        context: &Context<'context>,
    ) {
        let pointer = self.emit_storage_addr_of(slot, element_type, context);
        pointer.store(value, context);
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
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
            let declared_type = state_variable.get_type().ok_or_else(|| {
                anyhow::anyhow!(
                    "unresolved type for state variable: {}",
                    state_variable.name().name()
                )
            })?;
            let element_type = TypeConversion::resolve_slang_type(&declared_type, None, context);
            let address_type =
                Self::address_type(context, element_type, DataLocation::Storage, &declared_type);
            let storage_ref = Place::addr_of(&slot.name, address_type, context);
            let value = self.emit_value(&initializer, context)?;
            if declared_type.is_reference_type() {
                storage_ref.copy_from(value, context);
            } else {
                let stored_value =
                    TypeConversion::from_target_type(element_type, context).emit(value, context);
                storage_ref.store(stored_value, context);
            }
        }
        Ok(())
    }

    /// Returns a `!sol.ptr<element_type, Storage>` pointer via `sol.addr_of`.
    fn emit_storage_addr_of(
        &self,
        slot: &StorageSlot,
        element_type: Type<'context>,
        context: &Context<'context>,
    ) -> Place<'context> {
        let pointer_type = Type::pointer(context.melior, element_type, DataLocation::Storage);
        Place::addr_of(&slot.name, pointer_type, context)
    }
}
