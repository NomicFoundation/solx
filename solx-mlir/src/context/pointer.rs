//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the
//! loads, stores, and steps it supports.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::TypeAttribute;
use solx_utils::DataLocation;

use crate::Builder;
use crate::Type;
use crate::Value;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::GepOperation;
use crate::ods::sol::LoadOperation;
use crate::ods::sol::MapOperation;
use crate::ods::sol::StoreOperation;

/// A `!sol.ptr<T, Loc>` place: a typed address into stack / memory / storage /
/// calldata, carrying its pointee type and data location in its own MLIR type.
///
/// A newtype over the melior value (whose type is the pointer type) that is the
/// home for the place operations — load, store, and element stepping — mirroring
/// how [`Value`] homes the conversions a value undergoes. A `!sol.ptr` is itself
/// a first-class SSA value (a `storage` / `calldata` reference is the place, not
/// a loaded copy), so it converts to and from [`Value`] freely. The operations
/// take the [`Builder`] and the current block by parameter, exactly as a node's
/// emission does.
#[derive(Clone, Copy)]
pub struct Pointer<'context, 'block> {
    inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Pointer<'context, 'block> {
    /// Wraps a place value — a `!sol.ptr<…>`, or a by-reference aggregate in
    /// `Storage` / `CallData`, which is its own place ([`Type::address_type`]).
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

    /// The data location `Loc`.
    pub fn data_location(self) -> DataLocation {
        self.r#type().data_location()
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

    /// A stack slot default-initialised to the zero of `pointee`: a fresh
    /// zero-filled buffer for a memory aggregate, an empty buffer for `string` /
    /// `bytes`, the scalar/integer zero otherwise, and a bare slot for a
    /// reference the body binds before reading.
    pub fn default_initialized(
        pointee: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let slot = Self::stack_slot(pointee, builder, block);
        if pointee.is_string() {
            let buffer = Value::malloc(pointee.into_mlir(), false, builder, block);
            slot.store(buffer, builder, block);
        } else if (pointee.is_array() || pointee.is_struct())
            && matches!(pointee.data_location(), DataLocation::Memory)
        {
            let buffer = Value::malloc(pointee.into_mlir(), true, builder, block);
            slot.store(buffer, builder, block);
        } else if !pointee.is_reference() {
            slot.store(Value::zero(pointee, builder, block), builder, block);
        }
        slot
    }

    /// A stack slot of `pointee` seeded from the entry block's argument at
    /// `argument_index`. The block argument already carries the type, so the
    /// incoming value is spilled verbatim. The place each incoming parameter or
    /// threaded return value is spilled into.
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

    /// Loads the value of type `result_type` from this place (`sol.load`).
    /// Short-circuits when the place already *is* the value (the gep result for a
    /// reference-typed element in `Storage` / `CallData`), returning it unchanged.
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

    /// Steps to the place of an element / field of type `element_type` at `index`
    /// within this aggregate place (`sol.gep`). The result pointer type is derived
    /// from `(this pointer type, element_type)` by `sol::GepOp::getResultType` on
    /// the C++ side.
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

    /// Steps to the place of the mapping entry for `key` (`sol.map`). The dialect
    /// derives no map result type C-side (unlike [`gep`]), so the caller supplies
    /// the entry place type `entry_type` — the value type in the mapping's data
    /// location, or, for a reference value in `Storage` / `CallData`, the value
    /// type itself (the gep/map result-type rule returns a reference element
    /// unwrapped).
    ///
    /// [`gep`]: Self::gep
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
