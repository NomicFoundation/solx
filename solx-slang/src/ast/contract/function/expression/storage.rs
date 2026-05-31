//!
//! Storage load/store expression lowering via Sol dialect.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;

use ruint::aliases::U256;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use solx_utils::DataLocation;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a storage load via `sol.addr_of` + `sol.load`.
    pub(crate) fn emit_storage_load(
        &self,
        slot: U256,
        byte_offset: u32,
        element_type: Type<'context>,
        location: DataLocation,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let pointer = self.emit_storage_addr_of(slot, byte_offset, element_type, location, block);
        self.state
            .builder
            .emit_sol_load(pointer, element_type, block)
    }

    /// Emits a storage store via `sol.addr_of` + `sol.store`.
    pub(crate) fn emit_storage_store(
        &self,
        slot: U256,
        byte_offset: u32,
        value: Value<'context, 'block>,
        location: DataLocation,
        block: &BlockRef<'context, 'block>,
    ) {
        let pointer =
            self.emit_storage_addr_of(slot, byte_offset, value.r#type(), location, block);
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
        for state_variable in contract.compute_linearised_state_variables() {
            let Some(&(slot, byte_offset, location)) =
                self.storage_layout.get(&state_variable.node_id())
            else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            if matches!(initializer, Expression::ArrayExpression(_)) {
                anyhow::bail!("array-literal state variable initializers are not yet supported");
            }
            let declared_type = state_variable.get_type().ok_or_else(|| {
                anyhow::anyhow!(
                    "unresolved type for state variable: {}",
                    state_variable.name().name()
                )
            })?;
            let builder = &self.state.builder;
            let element_type = TypeConversion::resolve_slang_type(&declared_type, None, builder);
            let (value, next_block) = self.emit_value(&initializer, block)?;
            block = next_block;
            let address_type =
                Self::address_type(builder, element_type, location, &declared_type);
            let storage_ref = builder.emit_sol_addr_of(
                &crate::ast::contract::ContractEmitter::storage_symbol(slot, byte_offset, location),
                address_type,
                &block,
            );
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

    /// Returns a `!sol.ptr<element_type, location>` pointer via `sol.addr_of`,
    /// where `location` is `Storage` for persistent and `Transient` for
    /// transient state variables.
    fn emit_storage_addr_of(
        &self,
        slot: U256,
        byte_offset: u32,
        element_type: Type<'context>,
        location: DataLocation,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let pointer_type = self.state.builder.types.pointer(element_type, location);
        self.state.builder.emit_sol_addr_of(
            &crate::ast::contract::ContractEmitter::storage_symbol(slot, byte_offset, location),
            pointer_type,
            block,
        )
    }
}
