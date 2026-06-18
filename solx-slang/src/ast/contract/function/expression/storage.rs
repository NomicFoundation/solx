//!
//! Storage load/store expression emission via Sol dialect.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;

use solx_mlir::Builder;
use solx_mlir::ods::sol::AddrOfOperation;

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

    /// Returns the `!sol.ptr<element_type, location>` place via `sol.addr_of`.
    fn addr_of<'context, 'block>(
        &self,
        builder: &Builder<'context>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Pointer<'context, 'block>
    where
        'context: 'block,
    {
        let pointer_type =
            AstType::pointer(builder.context, element_type, self.location).into_mlir();
        Pointer::new(mlir_op!(
            builder,
            block,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(builder.context, &self.name))
                .addr(pointer_type)
        ))
    }
}
