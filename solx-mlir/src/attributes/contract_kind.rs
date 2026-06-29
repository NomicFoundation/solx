//!
//! Sol dialect contract kind attribute.
//!

sol_dialect_attribute! {
    /// Sol dialect contract kind (maps to the C++ `ContractKindAttr` values).
    ContractKind => crate::ffi::solxCreateContractKindAttr {
        /// Interface contract.
        Interface = 0,
        /// Regular contract.
        Contract = 1,
        /// Library contract.
        Library = 2,
    }
}
