//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use solx_utils::BIT_LENGTH_X64;

use crate::Builder;
use crate::CmpPredicate;
use crate::Type;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ConvCastOperation;
use crate::ods::sol::DefaultFuncConstantOperation;
use crate::ods::sol::GasLeftOperation;

/// An MLIR value in the Sol dialect.
///
/// A newtype over the melior value — which already carries its own MLIR type, so
/// the entity stays thin — that is the home for the conversions a value undergoes:
/// coercion, casting, and the truthiness test live on the value itself, not on a
/// context or a type-conversion god-object. The conversion methods take the
/// [`Builder`] and the current block by parameter, exactly as a node's emission
/// does.
#[derive(Clone, Copy)]
pub struct Value<'context, 'block> {
    inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Value<'context, 'block> {
    /// Wraps a melior value.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// Views a `!sol.ptr`-typed value as a [`Pointer`] place — the inverse of
    /// [`Pointer::into_value`]. The caller must ensure this value is a pointer.
    pub fn into_pointer(self) -> crate::Pointer<'context, 'block> {
        crate::Pointer::new(self.inner)
    }

    /// The value's type.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value —
    /// the common path for sizes, indices, and selectors whose magnitude is known
    /// to fit. Generic over the block because it is also emitted from inside the
    /// `Builder`'s own composite primitives, which carry a generic block.
    pub fn constant<B>(
        value: i64,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result_type = result_type.into_mlir();
        Self::new(sol_op!(
            builder,
            block,
            ConstantOperation
                .value(IntegerAttribute::new(result_type, value).into())
                .result(result_type)
        ))
    }

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`] — the path
    /// for literals that overflow `i64`. An `address` constant is built at the
    /// `ui160` address width and cast through the router; a boolean is an `i1`
    /// attribute; every other integer width is carried by the FFI big-integer
    /// attribute (the one primitive a `BigInt` constant cannot express through the
    /// fixed-width `IntegerAttribute`).
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if result_type == Type::address(builder.context, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                builder,
                block,
            );
            return integer.cast(result_type, builder, block);
        }
        let attribute: Attribute<'context> = if result_type.integer_bit_width()
            == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(result_type.into_mlir(), i64::from(*value != BigInt::ZERO)).into()
        } else {
            let (sign, words) = value.to_u64_digits();
            unsafe {
                Attribute::from_raw(crate::ffi::solxCreateIntegerAttr(
                    result_type.into_mlir().to_raw(),
                    sign == num::bigint::Sign::Minus,
                    words.len(),
                    words.as_ptr(),
                ))
            }
        };
        Self::new(sol_op!(
            builder,
            block,
            ConstantOperation
                .value(attribute)
                .result(result_type.into_mlir())
        ))
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean(
        value: bool,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN),
            builder,
            block,
        )
    }

    /// The zero of a scalar value type, built at its own representation width and
    /// bridged through the type's cast — never a narrowed wider constant (the
    /// `sol.cast` fold mishandles that). A UDVT arrives as its underlying type; a
    /// reference type is default-initialised through [`crate::Pointer`] instead.
    pub fn zero(
        r#type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if r#type.is_address() {
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_contract() {
            // address(0) reinterpreted as the contract (solc: ui160 -> address -> contract).
            let address = Self::zero(Type::address(builder.context, false), builder, block);
            address.cast(r#type, builder, block)
        } else if let Some(width) = r#type.fixed_bytes_or_byte_width() {
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, (width * 8) as usize),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_enum() {
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_ext_function_ref() {
            // A zero address + zero selector packed into the ext func ref.
            let address = Self::zero(Type::address(builder.context, false), builder, block);
            Self::new(builder.emit_sol_ext_func_constant(
                address.into_mlir(),
                0,
                r#type.into_mlir(),
                block,
            ))
        } else if r#type.is_function_ref() {
            // An internal pointer's zero reverts when called.
            Self::new(sol_op!(
                builder,
                block,
                DefaultFuncConstantOperation.addr(r#type.into_mlir())
            ))
        } else if IntegerType::try_from(r#type.into_mlir()).is_ok() {
            Self::constant(0, r#type, builder, block)
        } else {
            unreachable!("Value::zero handles only scalar value types")
        }
    }

    /// `sol.gasleft` — all remaining gas as a `ui256`. The default gas an
    /// external call forwards without an explicit `{gas: …}`, the gas of a bare
    /// low-level call, and the value of the `gasleft()` built-in: one home for
    /// the op rather than a Builder method and a built-in arm. Generic over the
    /// block because the call ops emit it from inside composite primitives.
    pub fn gas_left<B>(builder: &Builder<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(sol_op!(
            builder,
            block,
            GasLeftOperation
                .val(Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Coerces to `target_type`, emitting the conversion (nothing when the types
    /// already match). The single path every implicit widening and explicit
    /// `bool(x)` / `address(x)` / `uint(x)` takes: `bool(x)` is a truthiness
    /// test ([`Self::is_nonzero`]); every other target is a plain cast routed by
    /// the target type ([`Self::cast`]).
    pub fn coerce_to(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        if target_type == Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN) {
            return self.is_nonzero(builder, block);
        }
        self.cast(target_type, builder, block)
    }

    /// Casts to `target_type`, handing the value to the target type's cast router
    /// ([`Type::cast`]) — the kind-dispatch that selects the dialect cast op
    /// (`sol.cast` / `sol.bytes_cast` / `sol.address_cast` / …). Unlike
    /// [`Self::coerce_to`], a cast to `i1` is a plain representation cast, not a
    /// `bool(x)` truthiness test.
    pub fn cast(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        target_type.cast(self, builder, block)
    }

    /// Reinterprets the value's representation as `target_type` via
    /// `sol.conv_cast` — a representation-preserving cast the conversion pipeline
    /// rewrites to the remapped value. It crosses the inline-assembly boundary: a
    /// Solidity local's `!sol.ptr<T, Stack>` is reinterpreted as the `!llvm.ptr`
    /// that Yul `llvm.load`/`llvm.store` operate on. Returns the value unchanged
    /// when it already has `target_type`.
    pub fn reinterpret(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(sol_op!(
            builder,
            block,
            ConvCastOperation
                .inp(self.inner)
                .out(target_type.into_mlir())
        ))
    }

    /// Compares this value against `other` under `predicate` via `sol.cmp`,
    /// producing an `i1`.
    pub fn compare(
        self,
        other: Self,
        predicate: CmpPredicate,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let predicate_attribute = IntegerAttribute::new(
            IntegerType::new(builder.context, BIT_LENGTH_X64 as u32).into(),
            predicate as i64,
        );
        let value: MlirValue<'context, 'block> = sol_op!(
            builder,
            block,
            CmpOperation
                .predicate(predicate_attribute.into())
                .lhs(self.inner)
                .rhs(other.inner)
                .result(
                    Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir()
                )
        );
        Self::new(value)
    }

    /// Tests the value against zero, producing an `i1`. Short-circuits when the
    /// value is already `i1` (e.g. from a `sol.cmp`), avoiding a redundant
    /// `sol.cmp ne, %i1, %zero`.
    pub fn is_nonzero(
        self,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN) {
            return self;
        }
        let zero = Self::constant(0, self.r#type(), builder, block);
        self.compare(zero, CmpPredicate::Ne, builder, block)
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}
