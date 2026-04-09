//!
//! Sol dialect function kind attribute.
//!

/// Sol dialect function kind.
///
/// Maps to the `FunctionKindAttr` values in the C++ Sol dialect.
/// Regular functions do not carry a kind attribute.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// Constructor function.
    Constructor = 0,
    /// Fallback function.
    Fallback = 1,
    /// Receive function.
    Receive = 2,
}
