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
    /// `bool(x)` / `address(x)` / `uint(x)` takes: `i1` is a truthiness test,
    /// `address` truncates an integer through `ui160` then `address_cast`s,
    /// anything else is a plain `sol.cast`.
    pub fn coerce_to(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        let coerced = if target_type == builder.types.i1 {
            let zero = builder.emit_sol_constant(0, self.r#type(), block);
            self.compare(Self::new(zero), CmpPredicate::Ne, builder, block)
                .into_mlir()
        } else if target_type == builder.types.sol_address {
            let truncated = if IntegerType::try_from(self.r#type()).is_ok() {
                builder.emit_sol_cast(self.inner, builder.types.ui160, block)
            } else {
                self.inner
            };
            builder.emit_sol_address_cast(truncated, target_type, block)
        } else {
            builder.emit_sol_cast(self.inner, target_type, block)
        };
        Self::new(coerced)
    }

    /// A plain `sol.cast` to `target_type` — the integer-only conversion, without
    /// the truthiness / address special cases of [`Self::coerce_to`].
    pub fn cast(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(builder.emit_sol_cast(self.inner, target_type, block))
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
                .result(builder.types.i1)
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
        if self.r#type() == builder.types.i1 {
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
