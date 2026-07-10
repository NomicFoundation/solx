//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the loads, stores, and steps it supports.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;

use crate::Type;
use crate::Value;

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
