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

use crate::Context;
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
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let value_attribute =
            IntegerAttribute::try_from(Self::word_attribute(value, context.mlir()))
                .expect("yul.constant value is an i256 integer attribute");
        Self::new(mlir_op!(
            context,
            block,
            ConstantOperation
                .value(value_attribute)
                .out(Type::signless(context.mlir(), BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Loads a Yul word from an `!llvm.ptr` slot.
    pub fn load(
        pointer: MlirValue<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(
            block
                .append_operation(llvm::load(
                    context.mlir(),
                    pointer,
                    Type::signless(context.mlir(), BIT_LENGTH_FIELD).into_mlir(),
                    context.location(),
                    LoadStoreOptions::new().align(Some(Self::word_alignment(context))),
                ))
                .result(0)
                .expect("llvm.load always produces one result")
                .into(),
        )
    }

    /// Allocates a 256-bit `!llvm.ptr` stack slot for a Yul local.
    pub fn alloca(
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> MlirValue<'context, 'block> {
        let count = Self::constant(&BigInt::from(1u32), context, block);
        block
            .append_operation(llvm::alloca(
                context.mlir(),
                count.inner,
                Type::llvm_ptr(context.mlir()).into_mlir(),
                context.location(),
                AllocaOptions::new()
                    .align(Some(Self::word_alignment(context)))
                    .elem_type(Some(TypeAttribute::new(
                        Type::signless(context.mlir(), BIT_LENGTH_FIELD).into_mlir(),
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
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let predicate_attribute = predicate.attribute(context.mlir());
        Self::new(mlir_op!(
            context,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .out(Type::signless(context.mlir(), BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Stores this word into an `!llvm.ptr` slot.
    pub fn store(
        self,
        pointer: MlirValue<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        block.append_operation(llvm::store(
            context.mlir(),
            self.inner,
            pointer,
            context.location(),
            LoadStoreOptions::new().align(Some(Self::word_alignment(context))),
        ));
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// The `alignment = 32 : i64` attribute every Yul-word `llvm` slot op carries.
    fn word_alignment(context: &Context<'context>) -> IntegerAttribute<'context> {
        IntegerAttribute::new(
            IntegerType::new(context.mlir(), BIT_LENGTH_X64 as u32).into(),
            Self::WORD_ALIGNMENT,
        )
    }
}

impl<'context, 'block> IntoOds<MlirValue<'context, 'block>> for YulValue<'context, 'block> {
    fn into_ods(self) -> MlirValue<'context, 'block> {
        self.into_mlir()
    }
}
