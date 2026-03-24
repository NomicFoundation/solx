//!
//! Sol dialect state mutability attribute.
//!

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
