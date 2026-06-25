//!
//! Storage load/store expression emission via Sol dialect.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use solx_mlir::Builder;
use solx_mlir::mlir_op_build;
use solx_mlir::ods::sol::LoadImmutableOperation;
use solx_utils::DataLocation;

use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::storage_layout::StorageSlot;

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
        // An `immutable` has no storage address: it is read by symbol via `sol.load_immutable`,
        // matching solc (the constructor's write still goes through a `!sol.ptr<T, Immutable>` store).
        if matches!(self.location, DataLocation::Immutable) {
            let operation = block.append_operation(mlir_op_build!(
                builder,
                LoadImmutableOperation
                    ._name(FlatSymbolRefAttribute::new(builder.context, &self.name))
                    .val(element_type)
            ));
            return operation
                .result(0)
                .expect("sol.load_immutable produces one result")
                .into();
        }
        let pointer = self.addr_of(builder, element_type, block);
        pointer
            .load(AstType::new(element_type), builder, block)
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
        pointer.store(AstValue::new(value), builder, block);
    }

    /// Returns the place denoting this slot via `sol.addr_of`, typed by the element's `address_type`
    /// for the slot's location (a reference element addresses AS itself, so a later `load` short-circuits).
    fn addr_of<'context, 'block>(
        &self,
        builder: &Builder<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Pointer<'context, 'block>
    where
        'context: 'block,
    {
        let place_type = AstType::new(element_type).address_type(self.location, builder.context);
        Pointer::addr_of(&self.name, place_type, builder, block)
    }
}
