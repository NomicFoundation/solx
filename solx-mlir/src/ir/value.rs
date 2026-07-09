//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Attribute;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::IntegerAttribute;
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
    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`], selecting the dialect by target
    /// type: an address is built at `ui160` and cast; a boolean folds to `0`/`1`; every other integer
    /// takes the big-integer attribute.
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        if result_type == Type::address(context.melior, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                context,
            );
            return integer.address_cast(result_type, context);
        }
        let attribute: Attribute<'context> = if result_type.is_integer()
            && result_type.integer_bit_width() == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(result_type.into_mlir(), i64::from(value != &BigInt::ZERO)).into()
        } else {
            result_type.big_integer_attribute(value)
        };
        Self::from(mlir_op!(
            context,
            ConstantOperation
                .value(attribute)
                .result(result_type.into_mlir())
        ))
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
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN),
            context,
        )
    }

    /// Coerces to `target_type` per Solidity conversion semantics; a no-op at the target type.
    ///
    /// A boolean target compares against zero rather than bit-truncating; an address target
    /// truncates integers to ui160 before `sol.address_cast`; every other target is a `sol.cast`.
    pub fn coerce(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        if target_type == Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN) {
            return self.is_nonzero(context);
        }
        if target_type == Type::address(context.melior, false) {
            let truncated = if self.r#type().is_integer() {
                let ui160 = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS);
                self.cast(ui160, context)
            } else {
                self
            };
            return truncated.address_cast(target_type, context);
        }
        self.cast(target_type, context)
    }

    /// Compares against `other` under `predicate`, first coercing both operands to a common integer
    /// type: their shared type when equal, otherwise the 256-bit field.
    pub fn compare_coerced(
        self,
        other: Self,
        predicate: CmpPredicate,
        context: &Context<'context>,
    ) -> Self {
        let common_type = if self.r#type() == other.r#type() {
            self.r#type()
        } else {
            Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD)
        };
        let left = self.coerce(common_type, context);
        let right = other.coerce(common_type, context);
        left.compare(right, predicate, context)
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
