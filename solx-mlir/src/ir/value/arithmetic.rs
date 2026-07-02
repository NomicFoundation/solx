//!
//! Arithmetic, bitwise, and shift value producers.
//!
//! Binary operators read as instance methods on the left operand (`lhs.add(rhs, …)`); the result
//! type is inferred from `lhs` except where an op needs it stated (`exp`/`shl`/`shr`). `checked`
//! selects the reverting variant (`sol.cadd` etc.) and appears only where such a variant exists.
//! `addmod`/`mulmod` take three co-equal operands, so they read as constructors.
//!

use melior::ir::BlockLike;

use crate::Context;
use crate::Value;
use crate::ods::sol::AddModOperation;
use crate::ods::sol::AddOperation;
use crate::ods::sol::AndOperation;
use crate::ods::sol::CAddOperation;
use crate::ods::sol::CDivOperation;
use crate::ods::sol::CExpOperation;
use crate::ods::sol::CMulOperation;
use crate::ods::sol::CSubOperation;
use crate::ods::sol::DivOperation;
use crate::ods::sol::ExpOperation;
use crate::ods::sol::ModOperation;
use crate::ods::sol::MulModOperation;
use crate::ods::sol::MulOperation;
use crate::ods::sol::NotOperation;
use crate::ods::sol::OrOperation;
use crate::ods::sol::ShlOperation;
use crate::ods::sol::ShrOperation;
use crate::ods::sol::SubOperation;
use crate::ods::sol::XorOperation;

impl<'context, 'block> Value<'context, 'block> {
    /// Emits `sol.add` (`sol.cadd` when `checked`): `lhs + rhs`.
    pub fn add<B>(self, rhs: Self, checked: bool, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if checked {
            Self::new(mlir_op!(
                context,
                block,
                CAddOperation.lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::new(mlir_op!(
                context,
                block,
                AddOperation.lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.sub` (`sol.csub` when `checked`): `lhs - rhs`. Unary negation is `0 - x`.
    pub fn subtract<B>(
        self,
        rhs: Self,
        checked: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if checked {
            Self::new(mlir_op!(
                context,
                block,
                CSubOperation.lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::new(mlir_op!(
                context,
                block,
                SubOperation.lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.mul` (`sol.cmul` when `checked`): `lhs * rhs`.
    pub fn multiply<B>(
        self,
        rhs: Self,
        checked: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if checked {
            Self::new(mlir_op!(
                context,
                block,
                CMulOperation.lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::new(mlir_op!(
                context,
                block,
                MulOperation.lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.div` (`sol.cdiv` when `checked`): `lhs / rhs`.
    pub fn divide<B>(self, rhs: Self, checked: bool, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if checked {
            Self::new(mlir_op!(
                context,
                block,
                CDivOperation.lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::new(mlir_op!(
                context,
                block,
                DivOperation.lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.mod`: `lhs % rhs`. Modulo has no checked variant.
    pub fn remainder<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            ModOperation.lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.exp` (`sol.cexp` when `checked`): `lhs ** rhs`, result typed from `lhs`.
    pub fn exponentiate<B>(
        self,
        rhs: Self,
        checked: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result = self.r#type().into_mlir();
        if checked {
            Self::new(mlir_op!(
                context,
                block,
                CExpOperation.result(result).lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::new(mlir_op!(
                context,
                block,
                ExpOperation.result(result).lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.and`: `lhs & rhs`.
    pub fn bitand<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            AndOperation.lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.or`: `lhs | rhs`.
    pub fn bitor<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            OrOperation.lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.xor`: `lhs ^ rhs`.
    pub fn bitxor<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            XorOperation.lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.shl`: `lhs << rhs`, result typed from `lhs`.
    pub fn shl<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result = self.r#type().into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            ShlOperation.result(result).lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.shr`: `lhs >> rhs`, result typed from `lhs`.
    pub fn shr<B>(self, rhs: Self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result = self.r#type().into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            ShrOperation.result(result).lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.not`: `~self`.
    pub fn not<B>(self, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(context, block, NotOperation.value(self.inner)))
    }

    /// Emits `sol.addmod`: `(x + y) % modulus`.
    pub fn addmod<B>(
        x: Self,
        y: Self,
        modulus: Self,
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
            AddModOperation.x(x).y(y).r#mod(modulus)
        ))
    }

    /// Emits `sol.mulmod`: `(x * y) % modulus`.
    pub fn mulmod<B>(
        x: Self,
        y: Self,
        modulus: Self,
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
            MulModOperation.x(x).y(y).r#mod(modulus)
        ))
    }
}
