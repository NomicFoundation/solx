//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the loads, stores, and steps it supports.
//!

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
use crate::ods::sol::MallocOperation;
use crate::ods::sol::MapOperation;
use crate::ods::sol::StoreOperation;

/// A `!sol.ptr<T, Loc>` place: a typed address into stack / memory / storage / calldata, and the
/// load, store, and element-stepping operations it supports. A `!sol.ptr` is itself a first-class
/// SSA value: a `storage` / `calldata` reference is itself the place, so it converts to and from
/// [`Value`] freely.
#[derive(Clone, Copy)]
pub struct Place<'context> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'context>,
}

impl<'context> Place<'context> {
    /// Allocates a stack slot for `pointee` and returns the place: a
    /// `sol.alloca` yielding `!sol.ptr<pointee, Stack>`.
    pub fn stack(pointee: Type<'context>, context: &Context<'context>) -> Self {
        let address_type =
            Type::pointer(context.melior, pointee, solx_utils::DataLocation::Stack).into_mlir();
        Self::from(mlir_op!(
            context,
            AllocaOperation
                .alloc_type(TypeAttribute::new(address_type))
                .addr(address_type)
        ))
    }

    /// Allocates a fresh memory buffer typed as `pointee` and returns the place: a `sol.malloc`
    /// yielding the buffer, for a memory aggregate constructed via a literal.
    pub fn malloc(pointee: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, MallocOperation.addr(pointee.into_mlir())))
    }

    /// The place a named contract symbol denotes: `sol.addr_of @symbol` of `place_type`.
    pub fn addr_of(symbol: &str, place_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(context.melior, symbol))
                .addr(place_type.into_mlir())
        ))
    }

    /// Loads the value of `result_type` from this place (`sol.load`). Short-circuits when the
    /// place already *is* the value: a reference element in `Storage` / `CallData`.
    pub fn load(self, result_type: Type<'context>, context: &Context<'context>) -> Value<'context> {
        if self.r#type() == result_type {
            return self.into();
        }
        Value::from(mlir_op!(
            context,
            LoadOperation.addr(self.inner).out(result_type.into_mlir())
        ))
    }

    /// Stores `value` into this place (`sol.store`).
    pub fn store(self, value: Value<'context>, context: &Context<'context>) {
        mlir_op_void!(
            context,
            StoreOperation.val(value.into_mlir()).addr(self.inner)
        );
    }

    /// Deep-copies the reference `value`'s pointee into this place (`sol.copy`): the
    /// reference-to-reference counterpart of the scalar [`Self::store`].
    pub fn copy_from(self, value: Value<'context>, context: &Context<'context>) {
        mlir_op_void!(
            context,
            CopyOperation.src(value.into_mlir()).dst(self.inner)
        );
    }

    /// Steps to the place of element `element_type` at `index` within this aggregate place (`sol.gep`).
    /// The result place type comes from [`Type::gep_result_type`].
    pub fn gep(
        self,
        index: Value<'context>,
        element_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        let address_type = self.r#type().gep_result_type(element_type);
        Self::from(mlir_op!(
            context,
            GepOperation
                .base_addr(self.inner)
                .idx(index.into_mlir())
                .addr(address_type.into_mlir())
        ))
    }

    /// Steps to the place of the mapping entry for `key` (`sol.map`). The dialect derives no map
    /// result type C-side, so the caller supplies the entry place type `entry_type`.
    pub fn map(
        self,
        key: Value<'context>,
        entry_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        Self::from(mlir_op!(
            context,
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
    pub fn into_mlir(self) -> MlirValue<'context, 'context> {
        self.inner
    }
}

impl<'context, V> From<V> for Place<'context>
where
    V: ValueLike<'context>,
{
    /// Wraps a place value, laundering its block-scoped lifetime to `'context`.
    fn from(value: V) -> Self {
        Self {
            inner: unsafe { MlirValue::from_raw(value.to_raw()) },
        }
    }
}

impl<'context> From<Value<'context>> for Place<'context> {
    /// A `!sol.ptr` value is itself a place; both wrap the same SSA handle.
    fn from(value: Value<'context>) -> Self {
        Self {
            inner: value.into_mlir(),
        }
    }
}
