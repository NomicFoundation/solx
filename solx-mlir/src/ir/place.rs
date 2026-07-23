//!
//! A `!sol.ptr<T, Loc>` place in the Sol dialect: a typed address, and the loads, stores, and steps it supports.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;

use crate::Context;
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
    /// The place of the aggregate field at `field_index`: a `sol.gep` stepped by the index
    /// materialized as a `ui64` constant.
    pub fn gep_field(
        self,
        field_index: usize,
        field_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        self.gep(
            Value::constant(
                field_index as i64,
                Type::unsigned(context.melior, solx_utils::BIT_LENGTH_X64),
                context,
            ),
            field_type,
            context,
        )
    }

    /// Assigns `value` under the reference-vs-value idiom: a reference-typed place is its own address
    /// and takes a `sol.copy` that bridges type and data location, while a scalar slot stores the
    /// value converted to `element_type`. Yields the value the assignment expression evaluates to.
    pub fn assign(
        self,
        value: Value<'context>,
        element_type: Type<'context>,
        context: &Context<'context>,
    ) -> Value<'context> {
        if self.r#type() == element_type {
            self.copy_from(value, context);
            return Value::from(self);
        }
        let value = value.convert(element_type, context);
        self.store(value, context);
        value
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
