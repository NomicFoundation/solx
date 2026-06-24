//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the loads, stores, and steps it supports.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::TypeAttribute;
use solx_utils::DataLocation;

use crate::Builder;
use crate::Type;
use crate::Value;
use crate::ods::sol::AddrOfOperation;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::CopyOperation;
use crate::ods::sol::GepOperation;
use crate::ods::sol::LoadOperation;
use crate::ods::sol::MapOperation;
use crate::ods::sol::StoreOperation;

/// A `!sol.ptr<T, Loc>` place: a typed address into stack / memory / storage / calldata, and the
/// load, store, and element-stepping operations it supports. A `!sol.ptr` is itself a first-class
/// SSA value (a `storage` / `calldata` reference is the place), so it converts to and from [`Value`] freely.
#[derive(Clone, Copy)]
pub struct Pointer<'context, 'block> {
    inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Pointer<'context, 'block> {
    /// Wraps a place value — a `!sol.ptr<…>`, or a by-reference `Storage` / `CallData` aggregate (its own place).
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// The pointer as a [`Value`] — a `!sol.ptr` is a first-class SSA value.
    pub fn into_value(self) -> Value<'context, 'block> {
        Value::new(self.inner)
    }

    /// The pointer type `!sol.ptr<T, Loc>`.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// The pointee type `T`.
    pub fn pointee(self) -> Type<'context> {
        self.r#type().pointee()
    }

    /// Allocates a stack slot for `pointee` and returns the place — a
    /// `sol.alloca` yielding `!sol.ptr<pointee, Stack>`.
    pub fn stack_slot<B>(pointee: Type<'context>, builder: &Builder<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let address_type =
            Type::pointer(builder.context, pointee.into_mlir(), DataLocation::Stack).into_mlir();
        Self::new(mlir_op!(
            builder,
            block,
            AllocaOperation
                .alloc_type(TypeAttribute::new(address_type))
                .addr(address_type)
        ))
    }

    /// The place a named contract symbol denotes: `sol.addr_of @symbol` of `place_type`.
    pub fn addr_of<B>(
        symbol: &str,
        place_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            builder,
            block,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(builder.context, symbol))
                .addr(place_type.into_mlir())
        ))
    }

    /// A stack slot default-initialised to the zero of `pointee`.
    pub fn default_initialized(
        pointee: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let slot = Self::stack_slot(pointee, builder, block);
        if pointee.is_string() {
            unimplemented!("default-init of a string / bytes place is not yet supported")
        } else if (pointee.is_array() || pointee.is_struct())
            && matches!(pointee.data_location(), DataLocation::Memory)
        {
            unimplemented!("default-init of a memory aggregate place is not yet supported")
        } else if !pointee.is_reference() {
            slot.store(Value::zero(pointee, builder, block), builder, block);
        }
        slot
    }

    /// A stack slot of `pointee` seeded from the entry block's argument at `argument_index`.
    pub fn from_argument(
        pointee: Type<'context>,
        argument_index: usize,
        entry_block: &BlockRef<'context, 'block>,
        builder: &Builder<'context>,
    ) -> Self {
        let slot = Self::stack_slot(pointee, builder, entry_block);
        let argument = Value::new(
            entry_block
                .argument(argument_index)
                .expect("argument index is within the block signature")
                .into(),
        );
        slot.store(argument, builder, entry_block);
        slot
    }

    /// Loads the value of `result_type` from this place (`sol.load`). Short-circuits when the
    /// place already *is* the value (a reference element in `Storage` / `CallData`).
    pub fn load<B>(
        self,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == result_type {
            return self.into_value();
        }
        Value::new(mlir_op!(
            builder,
            block,
            LoadOperation.addr(self.inner).out(result_type.into_mlir())
        ))
    }

    /// Stores `value` into this place (`sol.store`).
    pub fn store<B>(self, value: Value<'context, 'block>, builder: &Builder<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(
            builder,
            block,
            StoreOperation.val(value.into_mlir()).addr(self.inner)
        );
    }

    /// Deep-copies the reference `value`'s pointee into this place (`sol.copy`) — the
    /// reference-to-reference counterpart of the scalar [`Self::store`].
    pub fn copy_from<B>(
        self,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(
            builder,
            block,
            CopyOperation.src(value.into_mlir()).dst(self.inner)
        );
    }

    /// Steps to the place of element `element_type` at `index` within this aggregate place (`sol.gep`).
    /// The result pointer type is derived C-side by `sol::GepOp::getResultType`.
    pub fn gep<B>(
        self,
        index: Value<'context, 'block>,
        element_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let address_type = unsafe {
            MlirType::from_raw(crate::ffi::mlirSolGepGetResultType(
                self.inner.r#type().to_raw(),
                element_type.into_mlir().to_raw(),
            ))
        };
        Self::new(mlir_op!(
            builder,
            block,
            GepOperation
                .base_addr(self.inner)
                .idx(index.into_mlir())
                .addr(address_type)
        ))
    }

    /// Steps to the place of the mapping entry for `key` (`sol.map`). The dialect derives no map
    /// result type C-side, so the caller supplies the entry place type `entry_type`.
    pub fn entry<B>(
        self,
        key: Value<'context, 'block>,
        entry_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            builder,
            block,
            MapOperation
                .mapping(self.inner)
                .key(key.into_mlir())
                .addr(entry_type.into_mlir())
        ))
    }
}
