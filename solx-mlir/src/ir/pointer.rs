//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the loads, stores, and steps it supports.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::TypeAttribute;

use crate::Context;
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
/// SSA value: a `storage` / `calldata` reference is itself the place, so it converts to and from
/// [`Value`] freely.
#[derive(Clone, Copy)]
pub struct Pointer<'context, 'block> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Pointer<'context, 'block> {
    /// Wraps a place value: a `!sol.ptr<...>`, or a by-reference `Storage` / `CallData` aggregate.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// Allocates a stack slot for `pointee` and returns the place: a
    /// `sol.alloca` yielding `!sol.ptr<pointee, Stack>`.
    pub fn stack<B>(pointee: Type<'context>, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let address_type = Type::pointer(
            context.mlir_context,
            pointee.into_mlir(),
            solx_utils::DataLocation::Stack,
        )
        .into_mlir();
        Self::new(mlir_op!(
            context,
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
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(context.mlir_context, symbol))
                .addr(place_type.into_mlir())
        ))
    }

    /// Loads the value of `result_type` from this place (`sol.load`). Short-circuits when the
    /// place already *is* the value: a reference element in `Storage` / `CallData`.
    pub fn load<B>(
        self,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == result_type {
            return self.into();
        }
        Value::new(mlir_op!(
            context,
            block,
            LoadOperation.addr(self.inner).out(result_type.into_mlir())
        ))
    }

    /// Stores `value` into this place (`sol.store`).
    pub fn store<B>(self, value: Value<'context, 'block>, context: &Context<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(
            context,
            block,
            StoreOperation.val(value.into_mlir()).addr(self.inner)
        );
    }

    /// Deep-copies the reference `value`'s pointee into this place (`sol.copy`): the
    /// reference-to-reference counterpart of the scalar [`Self::store`].
    pub fn copy_from<B>(self, value: Value<'context, 'block>, context: &Context<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(
            context,
            block,
            CopyOperation.src(value.into_mlir()).dst(self.inner)
        );
    }

    /// Steps to the place of element `element_type` at `index` within this aggregate place (`sol.gep`).
    /// The result place type comes from [`Type::gep_result_type`]. `no_panic_bounds` marks an index
    /// whose out-of-bounds access plain-reverts rather than raising `Panic(0x32)`.
    pub fn gep<B>(
        self,
        index: Value<'context, 'block>,
        element_type: Type<'context>,
        no_panic_bounds: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let address_type = self.r#type().gep_result_type(element_type);
        let mut gep = GepOperation::builder(context.mlir_context, context.location())
            .base_addr(self.inner)
            .idx(index.into_mlir())
            .addr(address_type.into_mlir());
        if no_panic_bounds {
            gep = gep.no_panic_bounds(Attribute::unit(context.mlir_context));
        }
        Self::new(
            block
                .append_operation(gep.build().into())
                .result(0)
                .expect("sol.gep produces one result")
                .into(),
        )
    }

    /// Steps to the place of the mapping entry for `key` (`sol.map`). The dialect derives no map
    /// result type C-side, so the caller supplies the entry place type `entry_type`.
    pub fn map<B>(
        self,
        key: Value<'context, 'block>,
        entry_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            MapOperation
                .mapping(self.inner)
                .key(key.into_mlir())
                .addr(entry_type.into_mlir())
        ))
    }

    /// The pointer type `!sol.ptr<T, Loc>`.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }
}

impl<'context, 'block> From<Value<'context, 'block>> for Pointer<'context, 'block> {
    /// A `!sol.ptr` value is itself a place; both wrap the same SSA handle.
    fn from(value: Value<'context, 'block>) -> Self {
        Self::new(value.into_mlir())
    }
}
