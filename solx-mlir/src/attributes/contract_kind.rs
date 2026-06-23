//!
//! Sol dialect contract kind attribute.
//!

use melior::Context;
use melior::ir::Attribute;

/// Sol dialect contract kind (maps to the C++ `ContractKindAttr` values).
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

impl ContractKind {
    /// Builds the Sol-dialect `ContractKindAttr` for this kind.
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateContractKindAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
