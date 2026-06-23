//!
//! Sol dialect function kind attribute.
//!

use melior::Context;
use melior::ir::Attribute;

/// Sol dialect function kind (maps to the C++ `FunctionKindAttr` values; regular functions carry none).
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
    /// Builds the Sol-dialect `FunctionKindAttr` for this kind.
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateFunctionKindAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
