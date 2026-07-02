//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

pub mod account;
pub mod arithmetic;
pub mod codec;
pub mod crypto;
pub mod environment;

use melior::ir::Attribute;
use melior::ir::BlockLike;
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
use crate::ods::sol::LengthOperation;
use crate::ods::sol::PopOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::StringLitOperation;

/// An MLIR value in the Sol dialect; the home for the conversions a value undergoes.
#[derive(Clone, Copy)]
pub struct Value<'context, 'block> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Value<'context, 'block> {
    /// Wraps a melior value.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// Wraps the `block` argument at `index`.
    pub fn argument<B>(block: &B, index: usize) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(
            block
                .argument(index)
                .expect("block argument index in range")
                .into(),
        )
    }

    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value.
    pub fn constant<B>(
        value: i64,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result_type = result_type.into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            ConstantOperation
                .value(Attribute::from(IntegerAttribute::new(result_type, value)))
                .result(result_type)
        ))
    }

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`], selecting the dialect by target
    /// type: an address is built at `ui160` and cast; a boolean folds to `0`/`1`; every other integer
    /// takes the big-integer attribute.
    pub fn constant_from_bigint<B>(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if result_type == Type::address(context.melior, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                context,
                block,
            );
            return integer.address_cast(result_type, context, block);
        }
        let attribute: Attribute<'context> = if result_type.is_integer()
            && result_type.integer_bit_width() == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(result_type.into_mlir(), i64::from(value != &BigInt::ZERO)).into()
        } else {
            result_type.big_integer_attribute(value)
        };
        Self::new(mlir_op!(
            context,
            block,
            ConstantOperation
                .value(attribute)
                .result(result_type.into_mlir())
        ))
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean<B>(value: bool, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN),
            context,
            block,
        )
    }

    /// A `string memory` value holding `text` verbatim, via `sol.string_lit`.
    pub fn string_literal<B>(text: &str, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            StringLitOperation
                .value(StringAttribute::new(context.melior, text))
                .addr(Type::string(context.melior, solx_utils::DataLocation::Memory).into_mlir())
        ))
    }

    /// `sol.array_lit`: an array of `array_type` constructed from `elements`.
    pub fn array_literal<B>(
        elements: &[Self],
        array_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let elements = elements
            .iter()
            .map(|element| element.into_mlir())
            .collect::<Vec<_>>();
        Self::new(mlir_op!(
            context,
            block,
            ArrayLitOperation
                .ins(elements.as_slice())
                .addr(array_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.cast`, a no-op when already that type.
    pub fn cast<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
            block,
            CastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.bytes_cast`, between byte / fixed-bytes / integer types; a
    /// no-op when already that type.
    pub fn bytes_cast<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
            block,
            BytesCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.address_cast`, between address and integer types.
    pub fn address_cast<B>(
        self,
        target_type: Type<'context>,
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
            AddressCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Appends a default element to this dynamic array / `bytes` value (`sol.push`), returning the
    /// reference to the newly appended slot of `slot_type`.
    pub fn push<B>(self, slot_type: Type<'context>, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            PushOperation.inp(self.inner).addr(slot_type.into_mlir())
        ))
    }

    /// Removes the last element of this dynamic array / `bytes` value (`sol.pop`).
    pub fn pop<B>(self, context: &Context<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(context, block, PopOperation.inp(self.inner));
    }

    /// Emits `sol.length`: the element count of the `aggregate` dynamic array / `bytes` value.
    pub fn length<B>(aggregate: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let field = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD).into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            LengthOperation.inp(aggregate).len(field)
        ))
    }

    /// Compares this value against `other` under `predicate` via `sol.cmp`, producing an `i1`.
    pub fn compare<B>(
        self,
        other: Self,
        predicate: CmpPredicate,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let predicate_attribute = predicate.attribute(context.melior);
        Self::new(mlir_op!(
            context,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .result(Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
        ))
    }

    /// The value's type.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}

impl<'context, 'block> From<Place<'context, 'block>> for Value<'context, 'block> {
    /// A `!sol.ptr` place is itself a first-class SSA value; both wrap the same handle.
    fn from(pointer: Place<'context, 'block>) -> Self {
        Self::new(pointer.into_mlir())
    }
}

impl<'context, 'block> IntoOds<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn into_ods(self) -> MlirValue<'context, 'block> {
        self.into_mlir()
    }
}
