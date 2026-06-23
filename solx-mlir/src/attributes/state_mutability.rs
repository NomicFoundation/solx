//!
//! Sol dialect state mutability attribute.
//!

use melior::Context;
use melior::ir::Attribute;
use slang_solidity_v2::ast::FunctionMutability;

/// Sol dialect state mutability (maps to the C++ `StateMutabilityAttr` values).
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

/// Maps Slang's `FunctionMutability` to the Sol dialect's `StateMutability`.
impl From<FunctionMutability> for StateMutability {
    fn from(mutability: FunctionMutability) -> Self {
        match mutability {
            FunctionMutability::Pure => Self::Pure,
            FunctionMutability::View => Self::View,
            FunctionMutability::Payable => Self::Payable,
            FunctionMutability::NonPayable => Self::NonPayable,
        }
    }
}

impl StateMutability {
    /// Builds the Sol-dialect `StateMutabilityAttr` for this mutability.
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateStateMutabilityAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
