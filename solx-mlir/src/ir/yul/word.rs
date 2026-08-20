//!
//! The EVM word every Yul value is, and the builtins that produce and consume it.
//!

use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;

use crate::IntoOds;

/// A signless `i256` SSA value in the Yul dialect: the single type Yul has. Distinct from
/// [`Value`](crate::Value), whose Sol dialect type carries the Solidity type and signedness a word
/// deliberately drops.
#[derive(Clone, Copy)]
pub struct Word<'context> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'context>,
}

impl<'context> Word<'context> {
    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'context> {
        self.inner
    }
}

impl<'context, V> From<V> for Word<'context>
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

impl<'context> IntoOds<MlirValue<'context, 'context>> for Word<'context> {
    fn into_ods(self) -> MlirValue<'context, 'context> {
        self.into_mlir()
    }
}
