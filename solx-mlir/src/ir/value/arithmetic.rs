//!
//! Arithmetic, bitwise, and shift value producers.
//!
//! Binary operators read as instance methods on the left operand (`lhs.add(rhs, …)`); the result
//! type is inferred from `lhs` except where an op needs it stated (`exp`/`shl`/`shr`). `checked`
//! selects the reverting variant (`sol.cadd` etc.) and appears only where such a variant exists.
//! `addmod`/`mulmod` take three co-equal operands, so they read as constructors.
//!

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

impl<'context> Value<'context> {
    /// Emits `sol.add` (`sol.cadd` when `checked`): `lhs + rhs`.
    pub fn add(self, rhs: Self, checked: bool, context: &Context<'context>) -> Self {
        if checked {
            Self::from(mlir_op!(context, CAddOperation.lhs(self.inner).rhs(rhs)))
        } else {
            Self::from(mlir_op!(context, AddOperation.lhs(self.inner).rhs(rhs)))
        }
    }

    /// Emits `sol.sub` (`sol.csub` when `checked`): `lhs - rhs`. Unary negation is `0 - x`.
    pub fn subtract(self, rhs: Self, checked: bool, context: &Context<'context>) -> Self {
        if checked {
            Self::from(mlir_op!(context, CSubOperation.lhs(self.inner).rhs(rhs)))
        } else {
            Self::from(mlir_op!(context, SubOperation.lhs(self.inner).rhs(rhs)))
        }
    }

    /// Emits `sol.mul` (`sol.cmul` when `checked`): `lhs * rhs`.
    pub fn multiply(self, rhs: Self, checked: bool, context: &Context<'context>) -> Self {
        if checked {
            Self::from(mlir_op!(context, CMulOperation.lhs(self.inner).rhs(rhs)))
        } else {
            Self::from(mlir_op!(context, MulOperation.lhs(self.inner).rhs(rhs)))
        }
    }

    /// Emits `sol.div` (`sol.cdiv` when `checked`): `lhs / rhs`.
    pub fn divide(self, rhs: Self, checked: bool, context: &Context<'context>) -> Self {
        if checked {
            Self::from(mlir_op!(context, CDivOperation.lhs(self.inner).rhs(rhs)))
        } else {
            Self::from(mlir_op!(context, DivOperation.lhs(self.inner).rhs(rhs)))
        }
    }

    /// Emits `sol.mod`: `lhs % rhs`. Modulo has no checked variant.
    pub fn remainder(self, rhs: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, ModOperation.lhs(self.inner).rhs(rhs)))
    }

    /// Emits `sol.exp` (`sol.cexp` when `checked`): `lhs ** rhs`, result typed from `lhs`.
    pub fn exponentiate(self, rhs: Self, checked: bool, context: &Context<'context>) -> Self {
        let result = self.r#type().into_mlir();
        if checked {
            Self::from(mlir_op!(
                context,
                CExpOperation.result(result).lhs(self.inner).rhs(rhs)
            ))
        } else {
            Self::from(mlir_op!(
                context,
                ExpOperation.result(result).lhs(self.inner).rhs(rhs)
            ))
        }
    }

    /// Emits `sol.and`: `lhs & rhs`.
    pub fn bitand(self, rhs: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, AndOperation.lhs(self.inner).rhs(rhs)))
    }

    /// Emits `sol.or`: `lhs | rhs`.
    pub fn bitor(self, rhs: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, OrOperation.lhs(self.inner).rhs(rhs)))
    }

    /// Emits `sol.xor`: `lhs ^ rhs`.
    pub fn bitxor(self, rhs: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, XorOperation.lhs(self.inner).rhs(rhs)))
    }

    /// Emits `sol.shl`: `lhs << rhs`, result typed from `lhs`.
    pub fn shl(self, rhs: Self, context: &Context<'context>) -> Self {
        let result = self.r#type().into_mlir();
        Self::from(mlir_op!(
            context,
            ShlOperation.result(result).lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.shr`: `lhs >> rhs`, result typed from `lhs`.
    pub fn shr(self, rhs: Self, context: &Context<'context>) -> Self {
        let result = self.r#type().into_mlir();
        Self::from(mlir_op!(
            context,
            ShrOperation.result(result).lhs(self.inner).rhs(rhs)
        ))
    }

    /// Emits `sol.not`: `~self`.
    pub fn not(self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, NotOperation.value(self.inner)))
    }

    /// Emits `sol.addmod`: `(x + y) % modulus`.
    pub fn addmod(x: Self, y: Self, modulus: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, AddModOperation.x(x).y(y).r#mod(modulus)))
    }

    /// Emits `sol.mulmod`: `(x * y) % modulus`.
    pub fn mulmod(x: Self, y: Self, modulus: Self, context: &Context<'context>) -> Self {
        Self::from(mlir_op!(context, MulModOperation.x(x).y(y).r#mod(modulus)))
    }
}
