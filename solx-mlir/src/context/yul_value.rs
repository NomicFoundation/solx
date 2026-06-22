//!
//! A Yul-dialect value: the untyped 256-bit word and its local-slot primitives.
//!

use melior::dialect::llvm;
use melior::dialect::llvm::AllocaOptions;
use melior::dialect::llvm::LoadStoreOptions;
use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use solx_utils::BIT_LENGTH_FIELD;
use solx_utils::BIT_LENGTH_X64;

use crate::Builder;
use crate::IntoOds;
use crate::Type;
use crate::YulCmpPredicate;
use crate::ods::yul::*;

/// A Yul value — the signless `i256` word every inline-assembly computation produces;
/// the peer of [`crate::Value`] for the untyped Yul dialect.
#[derive(Clone, Copy)]
pub struct YulValue<'context, 'block> {
    inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> YulValue<'context, 'block> {
    /// The byte alignment `solc` emits on every Yul-word `llvm.alloca`/`load`/`store`.
    const WORD_ALIGNMENT: i64 = 32;

    /// Wraps a melior value known to be a Yul word.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// The signless `i256` integer attribute for a Yul word, via the FFI big-integer attribute (exceeds `i64`).
    pub fn word_attribute(
        value: &BigInt,
        context: &'context melior::Context,
    ) -> Attribute<'context> {
        let (sign, words) = value.to_u64_digits();
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateIntegerAttr(
                Type::signless(context, BIT_LENGTH_FIELD)
                    .into_mlir()
                    .to_raw(),
                sign == num::bigint::Sign::Minus,
                words.len(),
                words.as_ptr(),
            ))
        }
    }

    /// `yul.constant` materialising the 256-bit word `value`.
    pub fn constant(
        value: &BigInt,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let value_attribute =
            IntegerAttribute::try_from(Self::word_attribute(value, builder.context))
                .expect("yul.constant value is an i256 integer attribute");
        Self::new(mlir_op!(
            builder,
            block,
            ConstantOperation
                .value(value_attribute)
                .out(Type::signless(builder.context, BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Loads a Yul word from an `!llvm.ptr` slot.
    pub fn load(
        pointer: MlirValue<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(
            block
                .append_operation(llvm::load(
                    builder.context,
                    pointer,
                    Type::signless(builder.context, BIT_LENGTH_FIELD).into_mlir(),
                    builder.unknown_location,
                    LoadStoreOptions::new().align(Some(Self::word_alignment(builder))),
                ))
                .result(0)
                .expect("llvm.load always produces one result")
                .into(),
        )
    }

    /// Allocates a 256-bit `!llvm.ptr` stack slot for a Yul local.
    pub fn alloca(
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> MlirValue<'context, 'block> {
        let count = Self::constant(&BigInt::from(1u32), builder, block);
        block
            .append_operation(llvm::alloca(
                builder.context,
                count.inner,
                Type::llvm_ptr(builder.context).into_mlir(),
                builder.unknown_location,
                AllocaOptions::new()
                    .align(Some(Self::word_alignment(builder)))
                    .elem_type(Some(TypeAttribute::new(
                        Type::signless(builder.context, BIT_LENGTH_FIELD).into_mlir(),
                    ))),
            ))
            .result(0)
            .expect("llvm.alloca always produces one result")
            .into()
    }

    /// Compares against `other` under `predicate` via `yul.cmp`, producing the
    /// word `1` or `0`.
    pub fn compare(
        self,
        other: Self,
        predicate: YulCmpPredicate,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let predicate_attribute = predicate.attribute(builder.context);
        Self::new(mlir_op!(
            builder,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .out(Type::signless(builder.context, BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Stores this word into an `!llvm.ptr` slot.
    pub fn store(
        self,
        pointer: MlirValue<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        block.append_operation(llvm::store(
            builder.context,
            self.inner,
            pointer,
            builder.unknown_location,
            LoadStoreOptions::new().align(Some(Self::word_alignment(builder))),
        ));
    }

    /// The `alignment = 32 : i64` attribute every Yul-word `llvm` slot op carries.
    fn word_alignment(builder: &Builder<'context>) -> IntegerAttribute<'context> {
        IntegerAttribute::new(
            IntegerType::new(builder.context, BIT_LENGTH_X64 as u32).into(),
            Self::WORD_ALIGNMENT,
        )
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for YulValue<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}

impl<'context, 'block> IntoOds<MlirValue<'context, 'block>> for YulValue<'context, 'block> {
    fn into_ods(self) -> MlirValue<'context, 'block> {
        self.into_mlir()
    }
}
