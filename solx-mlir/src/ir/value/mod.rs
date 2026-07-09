//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

pub mod account;
pub mod arithmetic;
pub mod codec;
pub mod crypto;
pub mod environment;

use melior::ir::Attribute;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use num::BigInt;

use crate::CmpPredicate;
use crate::Context;
use crate::IntoOds;
use crate::Place;
use crate::Type;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::ArrayLitOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::FixedBytesIndexOperation;
use crate::ods::sol::LengthOperation;
use crate::ods::sol::PopOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::StringLitOperation;

/// An MLIR value in the Sol dialect; the home for the conversions a value undergoes.
#[derive(Clone, Copy)]
pub struct Value<'context> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'context>,
}

impl<'context> Value<'context> {
    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value.
    pub fn constant(value: i64, result_type: Type<'context>, context: &Context<'context>) -> Self {
        let result_type = result_type.into_mlir();
        Self::from(mlir_op!(
            context,
            ConstantOperation
                .value(Attribute::from(IntegerAttribute::new(result_type, value)))
                .result(result_type)
        ))
    }

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

    /// A `string memory` value holding `text` verbatim, via `sol.string_lit`.
    pub fn string_literal(text: &str, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            StringLitOperation
                .value(StringAttribute::new(context.melior, text))
                .addr(Type::string(context.melior, solx_utils::DataLocation::Memory).into_mlir())
        ))
    }

    /// `sol.array_lit`: an array of `array_type` constructed from `elements`.
    pub fn array_literal(
        elements: &[Self],
        array_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        let elements = elements
            .iter()
            .map(|element| element.into_mlir())
            .collect::<Vec<_>>();
        Self::from(mlir_op!(
            context,
            ArrayLitOperation
                .ins(elements.as_slice())
                .addr(array_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.cast`, a no-op when already that type.
    pub fn cast(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        Self::from(mlir_op!(
            context,
            CastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.bytes_cast`, between byte / fixed-bytes / integer types; a
    /// no-op when already that type.
    pub fn bytes_cast(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        Self::from(mlir_op!(
            context,
            BytesCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.address_cast`, between address and integer types.
    pub fn address_cast(self, target_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            AddressCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
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

    /// Extracts the `index`-th byte of this fixed-bytes value as `bytes1` via `sol.fixed_bytes_index`.
    pub fn fixed_bytes_index(self, index: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            FixedBytesIndexOperation
                .value(self.inner)
                .index(index.inner)
                .result(Type::fixed_bytes(context.melior, 1).into_mlir())
        ))
    }

    /// Appends a default element to this dynamic array / `bytes` value (`sol.push`), returning the
    /// reference to the newly appended slot of `slot_type`.
    pub fn push(self, slot_type: Type<'context>, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(
            context,
            PushOperation.inp(self.inner).addr(slot_type.into_mlir())
        ))
    }

    /// Removes the last element of this dynamic array / `bytes` value (`sol.pop`).
    pub fn pop(self, context: &Context<'context>) {
        mlir_op_void!(context, PopOperation.inp(self.inner));
    }

    /// Emits `sol.length`: the element count of the `aggregate` dynamic array / `bytes` value.
    pub fn length(aggregate: Self, context: &Context<'context>) -> Self {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::from(mlir_op!(context, LengthOperation.inp(aggregate).len(field)))
    }

    /// Compares this value against `other` under `predicate` via `sol.cmp`, producing an `i1`.
    pub fn compare(
        self,
        other: Self,
        predicate: CmpPredicate,
        context: &Context<'context>,
    ) -> Self {
        let predicate_attribute = predicate.attribute(context.melior);
        Self::from(mlir_op!(
            context,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .result(Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
        ))
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
