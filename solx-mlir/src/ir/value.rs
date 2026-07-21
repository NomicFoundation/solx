//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use num::BigInt;

use crate::CmpPredicate;
use crate::Context;
use crate::IntoOds;
use crate::Place;
use crate::Type;
use crate::ods::sol::ConstantOperation;

/// An MLIR value in the Sol dialect; the home for the conversions a value undergoes.
#[derive(Clone, Copy)]
pub struct Value<'context> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'context>,
}

impl<'context> Value<'context> {
    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`] at the type a constant can be
    /// emitted at — `ui160` for an address, the target type itself otherwise — and converts it to the
    /// target.
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        let r#type = if result_type.is_address() {
            Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS)
        } else {
            result_type
        };
        Self::from(mlir_op!(
            context,
            ConstantOperation
                .value(r#type.integer_attribute(value))
                .result(r#type.into_mlir())
        ))
        .convert(result_type, context)
    }

    /// Materialises the additive identity `0` of `result_type`.
    pub fn zero(result_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::constant(0, result_type, context)
    }

    /// Materialises the multiplicative identity `1` of `result_type`.
    pub fn one(result_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::constant(1, result_type, context)
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean(value: bool, context: &Context<'context>) -> Self {
        Self::constant(i64::from(value), Type::boolean(context.melior), context)
    }

    /// Coerces to `target_type` under Solidity's implicit conversions; a no-op at the target type. A
    /// bytes-like target reinterprets via `sol.bytes_cast`; every other target is a `sol.cast`.
    pub fn coerce(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        if target_type.is_bytes_like() {
            return self.bytes_cast(target_type, context);
        }
        self.cast(target_type, context)
    }

    /// Converts to `target_type` under an explicit `T(x)` cast the binder has admitted. An address
    /// target truncates an integer operand to `ui160` before `sol.address_cast` — the conversion
    /// Solidity forbids implicitly, hence its home here rather than in `coerce`; every other target
    /// shares `coerce`'s dispatch.
    pub fn convert(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if target_type.is_address() {
            let truncated = if self.r#type().is_integer() {
                let ui160 = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS);
                self.cast(ui160, context)
            } else {
                self
            };
            return truncated.address_cast(target_type, context);
        }
        self.coerce(target_type, context)
    }

    /// The `i1` truthiness of `self` via `sol.cmp ne 0`; a no-op when `self` is already `i1`.
    pub fn is_nonzero(self, context: &Context<'context>) -> Self {
        if self.r#type().is_integer()
            && self.r#type().integer_bit_width() == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            return self;
        }
        let zero = Self::zero(self.r#type(), context);
        self.compare(zero, CmpPredicate::Ne, context)
    }

    /// The value's type.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'context> {
        self.inner
    }
}

impl<'context, V> From<V> for Value<'context>
where
    V: ValueLike<'context>,
{
    /// Wraps a melior value, laundering its block-scoped lifetime to `'context`.
    fn from(value: V) -> Self {
        Self {
            inner: unsafe { MlirValue::from_raw(value.to_raw()) },
        }
    }
}

impl<'context> From<Place<'context>> for Value<'context> {
    /// A `!sol.ptr` place is itself a first-class SSA value; both wrap the same handle.
    fn from(pointer: Place<'context>) -> Self {
        Self {
            inner: pointer.into_mlir(),
        }
    }
}

impl<'context> IntoOds<MlirValue<'context, 'context>> for Value<'context> {
    fn into_ods(self) -> MlirValue<'context, 'context> {
        self.into_mlir()
    }
}
