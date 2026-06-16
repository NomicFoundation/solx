//!
//! Sol dialect function kind attribute.
//!

use melior::Context;
use melior::ir::Attribute;

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

impl FunctionKind {
    /// Builds the Sol-dialect `FunctionKindAttr` for this kind — the dialect
    /// representation a `sol.func` carries (a regular function carries none).
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        // `solxCreateFunctionKindAttr` returns a valid MlirAttribute.
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateFunctionKindAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
