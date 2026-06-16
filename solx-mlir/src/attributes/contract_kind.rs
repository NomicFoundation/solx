//!
//! Sol dialect contract kind attribute.
//!

use melior::Context;
use melior::ir::Attribute;

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

impl ContractKind {
    /// Builds the Sol-dialect `ContractKindAttr` for this kind — the dialect
    /// representation a `sol.contract` carries, owned by the kind itself rather
    /// than spelled at each emission site.
    pub fn attribute(self, context: &Context) -> Attribute<'_> {
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateContractKindAttr(
                context.to_raw(),
                self as u32,
            ))
        }
    }
}
