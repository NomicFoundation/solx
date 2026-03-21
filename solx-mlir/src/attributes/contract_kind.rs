//!
//! Sol dialect contract kind attribute.
//!

/// Sol dialect contract kind.
///
/// Maps to the `ContractKindAttr` values in the C++ Sol dialect.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    /// Interface contract.
    Interface = 0,
    /// Regular contract.
    Contract = 1,
    /// Library contract.
    Library = 2,
}
