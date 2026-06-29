//!
//! Sol dialect function kind attribute.
//!

sol_dialect_attribute! {
    /// Sol dialect function kind (maps to the C++ `FunctionKindAttr` values; regular functions carry none).
    FunctionKind => crate::ffi::solxCreateFunctionKindAttr {
        /// Constructor function.
        Constructor = 0,
        /// Fallback function.
        Fallback = 1,
        /// Receive function.
        Receive = 2,
    }
}
