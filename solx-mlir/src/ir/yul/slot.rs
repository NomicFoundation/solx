//!
//! A `!yul.ptr` slot: the address a Yul variable's word is held at.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;

use crate::IntoOds;

/// A `!yul.ptr`: the address of one EVM word. Every Yul variable - a `let` binding, a function
/// parameter, a function's return variable - is held in a slot rather than an SSA value, because Yul
/// assigns to a name after binding it. A Solidity variable an assembly block reaches also surfaces
/// as a slot, through the `sol.yul_*` bridge ops on [`Place`](crate::Place).
#[derive(Clone, Copy)]
pub struct Slot<'context> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'context>,
}

impl<'context> Slot<'context> {
    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'context> {
        self.inner
    }
}

impl<'context, V> From<V> for Slot<'context>
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

impl<'context> IntoOds<MlirValue<'context, 'context>> for Slot<'context> {
    fn into_ods(self) -> MlirValue<'context, 'context> {
        self.into_mlir()
    }
}
