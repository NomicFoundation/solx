//!
//! Storage load/store expression emission via Sol dialect.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use solx_mlir::Context;
use solx_mlir::ods::sol::LoadImmutableOperation;

use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::storage_layout::StorageSlot;

impl StorageSlot {
    /// Emits a load of this slot via `sol.addr_of` + `sol.load`.
    pub fn load<'context, 'block>(
        &self,
        context: &Context<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        if matches!(self.location, solx_utils::DataLocation::Immutable) {
            return mlir_op!(
                context,
                block,
                LoadImmutableOperation
                    ._name(FlatSymbolRefAttribute::new(
                        context.mlir_context,
                        &self.name
                    ))
                    .val(element_type)
            );
        }
        let pointer = self.addr_of(context, element_type, block);
        pointer
            .load(AstType::new(element_type), context, block)
            .into_mlir()
    }

    /// Emits a store into this slot via `sol.addr_of` + `sol.store`.
    pub fn store<'context, 'block>(
        &self,
        context: &Context<'context>,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) where
        'context: 'block,
    {
        let pointer = self.addr_of(context, element_type, block);
        pointer.store(AstValue::new(value), context, block);
    }

    /// Returns the place denoting this slot via `sol.addr_of`, typed by the element's `address_type`
    /// for the slot's location.
    fn addr_of<'context, 'block>(
        &self,
        context: &Context<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Pointer<'context, 'block>
    where
        'context: 'block,
    {
        let place_type =
            AstType::new(element_type).address_type(self.location, context.mlir_context);
        Pointer::addr_of(&self.name, place_type, context, block)
    }
}
