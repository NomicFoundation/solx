//!
//! A value produced during emission, and the conversions it undergoes.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::r#type::IntegerType;
use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::CmpOperation;
use solx_mlir::ods::sol::ConvCastOperation;
use solx_utils::BIT_LENGTH_X64;

/// An MLIR value produced during emission.
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

    /// The value's MLIR type.
    pub fn r#type(self) -> Type<'context> {
        self.inner.r#type()
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
        if target_type
            == crate::ast::Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN)
                .into_mlir()
        {
            return self.is_nonzero(builder, block);
        }
        self.cast(target_type, builder, block)
    }

    /// Casts to `target_type`, handing the value to the target type's cast router
    /// ([`crate::ast::Type::cast`]) — the kind-dispatch that selects the dialect
    /// cast op (`sol.cast` / `sol.bytes_cast` / `sol.address_cast` / …). Unlike
    /// [`Self::coerce_to`], a cast to `i1` is a plain representation cast, not a
    /// `bool(x)` truthiness test.
    pub fn cast(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        crate::ast::Type::new(target_type).cast(self, builder, block)
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
            ConvCastOperation.inp(self.inner).out(target_type)
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
                    crate::ast::Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN)
                        .into_mlir()
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
        if self.r#type()
            == crate::ast::Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN)
                .into_mlir()
        {
            return self;
        }
        let zero = builder.emit_sol_constant(0, self.r#type(), block);
        self.compare(Self::new(zero), CmpPredicate::Ne, builder, block)
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}
