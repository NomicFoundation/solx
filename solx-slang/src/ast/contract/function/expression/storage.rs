//!
//! Storage load/store expression emission via Sol dialect.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use slang_solidity_v2::ast::ContractDefinition;
use solx_mlir::Builder;
use solx_mlir::ods::sol::AddrOfOperation;
use solx_mlir::ods::sol::CopyOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;

impl StorageSlot {
    /// Emits a load of this slot via `sol.addr_of` + `sol.load`.
    pub fn load<'context, 'block>(
        &self,
        builder: &Builder<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        let pointer = self.addr_of(builder, element_type, block);
        pointer
            .load(crate::ast::Type::new(element_type), builder, block)
            .into_mlir()
    }

    /// Emits a store into this slot via `sol.addr_of` + `sol.store`.
    pub fn store<'context, 'block>(
        &self,
        builder: &Builder<'context>,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) where
        'context: 'block,
    {
        let pointer = self.addr_of(builder, element_type, block);
        pointer.store(crate::ast::Value::new(value), builder, block);
    }

    /// Returns the `!sol.ptr<element_type, location>` place via `sol.addr_of`.
    fn addr_of<'context, 'block>(
        &self,
        builder: &Builder<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> crate::ast::Pointer<'context, 'block>
    where
        'context: 'block,
    {
        let pointer_type =
            crate::ast::Type::pointer(builder.context, element_type, self.location).into_mlir();
        crate::ast::Pointer::new(sol_op!(
            builder,
            block,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(builder.context, &self.name))
                .addr(pointer_type)
        ))
    }
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
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
        // Run initializers for the whole C3-linearised hierarchy (inherited +
        // own) in linearisation order, so a derived contract's construction
        // executes its base contracts' state-variable initializers — including
        // their side effects (`uint y = f();`) — exactly as solc does.
        for state_variable in contract.linearised_state_variables() {
            let Some(slot) = self.storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            let declared_type = state_variable
                .get_type()
                .expect("slang types every state variable");
            let builder = &self.state.builder;
            let element_type = declared_type.resolve_type(LocationPolicy::Declared(None), builder);
            let address_type =
                Self::address_type(builder, element_type, slot.location, &declared_type);
            let storage_ref = sol_op!(
                builder,
                &block,
                AddrOfOperation
                    .var(FlatSymbolRefAttribute::new(builder.context, &slot.name))
                    .addr(address_type)
            );
            let BlockAnd {
                value,
                block: next_block,
            } = initializer.emit(self, block);
            block = next_block;
            if declared_type.is_reference_type() {
                sol_op_void!(
                    builder,
                    &block,
                    CopyOperation.src(value.into_mlir()).dst(storage_ref)
                );
            } else {
                let stored_value =
                    value.coerce_to(crate::ast::Type::new(element_type), builder, &block);
                crate::ast::Pointer::new(storage_ref).store(stored_value, builder, &block);
            }
        }
        block
    }
}
