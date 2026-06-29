//!
//! Sol dialect state mutability attribute.
//!

use slang_solidity_v2::ast::FunctionMutability;

sol_u32_attribute! {
    /// Sol dialect state mutability (maps to the C++ `StateMutabilityAttr` values).
    StateMutability => crate::ffi::solxCreateStateMutabilityAttr {
        /// Pure — no reads or writes.
        Pure = 0,
        /// View — reads state, no writes.
        View = 1,
        /// NonPayable — reads/writes state, no ether.
        NonPayable = 2,
        /// Payable — can receive ether.
        Payable = 3,
    }
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
