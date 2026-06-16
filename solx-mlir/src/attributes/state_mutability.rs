//!
//! Sol dialect state mutability attribute.
//!

use melior::Context;
use melior::ir::Attribute;

/// Sol dialect state mutability.
///
/// Maps to the `StateMutabilityAttr` values in the C++ Sol dialect.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMutability {
    /// Pure — no reads or writes.
    Pure = 0,
    /// View — reads state, no writes.
    View = 1,
    /// NonPayable — reads/writes state, no ether.
    NonPayable = 2,
    /// Payable — can receive ether.
    Payable = 3,
}

impl StateMutability {
    /// Builds the Sol-dialect `StateMutabilityAttr` for this mutability — the
    /// dialect representation a `sol.func` carries, owned by the mutability rather
    /// than spelled at the emission site.
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        // `solxCreateStateMutabilityAttr` returns a valid MlirAttribute.
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateStateMutabilityAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
