//!
//! Sol dialect state mutability encoding.
//!

/// Sol dialect state mutability encoding.
///
/// Maps to the `StateMutabilityAttr` values in the C++ Sol dialect.
#[repr(u32)]
pub(crate) enum StateMutability {
    /// Pure — no reads or writes.
    Pure = 0,
    /// View — reads state, no writes.
    View = 1,
    /// NonPayable — reads/writes state, no ether.
    NonPayable = 2,
    /// Payable — can receive ether.
    Payable = 3,
}
