//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use num::BigInt;
use num::One;
use num::Zero;
use num::bigint::Sign;

use solx_utils::DataLocation;

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
    /// emitted at, then converts it to the target: `ui160` for an address, the width-matched unsigned
    /// integer for a bytes-like target, the target type itself otherwise.
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        let r#type = if result_type.is_address() {
            Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS)
        } else if result_type.is_bytes_like() {
            let bits = result_type.bytes_like_width() as usize * solx_utils::BIT_LENGTH_BYTE;
            Type::unsigned(context.melior, bits)
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

    /// Materialises the zero of `result_type`, the default value a Solidity value type carries.
    pub fn zero(result_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::constant_from_bigint(&BigInt::zero(), result_type, context)
    }

    /// Materialises the multiplicative identity `1` of `result_type`.
    pub fn one(result_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::constant_from_bigint(&BigInt::one(), result_type, context)
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean(value: bool, context: &Context<'context>) -> Self {
        Self::constant(i64::from(value), Type::boolean(context.melior), context)
    }

    /// Materialises the default-initialized value of `target_type`, matching solc's default-init:
    /// a scalar's `zero`; an allocated empty memory `bytes`/`string`; a zero-filled memory array
    /// or struct; the `sol.default_storage` / `sol.default_calldata` designator for a storage or
    /// calldata reference.
    pub fn default_initialized(target_type: Type<'context>, context: &Context<'context>) -> Self {
        if target_type.is_scalar() {
            return Self::zero(target_type, context);
        }
        match target_type.data_location() {
            DataLocation::Memory if target_type.is_string() => {
                Place::malloc(target_type, context).into()
            }
            DataLocation::Memory => Place::malloc_zeroed(target_type, context).into(),
            DataLocation::Storage => Place::default_storage(target_type, context).into(),
            DataLocation::CallData => Place::default_calldata(target_type, context).into(),
            other => unreachable!(
                "a reference default-initializes in memory, storage, or calldata; got {other:?}"
            ),
        }
    }

    /// Materialises the `bytesN` constant of a string or hex literal: `bytes` left-aligned in
    /// `target_type`'s width, right-zero-padded, built as the unsigned integer of that width and
    /// reinterpreted with `sol.bytes_cast`.
    pub fn left_aligned_bytes(
        mut bytes: Vec<u8>,
        target_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        bytes.resize(target_type.bytes_like_width() as usize, 0);
        let integer = Self::constant_from_bigint(
            &BigInt::from_bytes_be(Sign::Plus, &bytes),
            Type::unsigned(context.melior, bytes.len() * solx_utils::BIT_LENGTH_BYTE),
            context,
        );
        integer.bytes_cast(target_type, context)
    }

    /// Emits the cast reconciling `self` to `target_type`, be it a computed common type or an
    /// explicit `T(x)`. The address and string-to-`bytesN` arms, reachable only under an explicit
    /// cast, precede the bytes and scalar arms an address or string would otherwise fall into.
    pub fn convert(mut self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        if self.r#type().is_address() {
            return self.address_cast(target_type, context);
        }
        if target_type.is_address() {
            if self.r#type().is_integer() {
                self = self.cast(
                    Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                    context,
                );
            }
            return self.address_cast(target_type, context);
        }
        if self.r#type().is_string() && target_type.is_bytes_like() {
            return self.dyn_bytes_to_fixedbytes(target_type, context);
        }
        if self.r#type().is_bytes_like() {
            return self.bytes_cast(target_type, context);
        }
        if target_type.is_bytes_like() {
            let integer_type = Type::unsigned(
                context.melior,
                target_type.bytes_like_width() as usize * solx_utils::BIT_LENGTH_BYTE,
            );
            return self
                .cast(integer_type, context)
                .bytes_cast(target_type, context);
        }
        if !self.r#type().is_scalar() && !target_type.is_scalar() {
            return self.data_loc_cast(target_type, context);
        }
        self.cast(target_type, context)
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
