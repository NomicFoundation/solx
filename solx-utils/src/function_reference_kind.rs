//!
//! Sol dialect dispatch for function reference types.
//!

/// Sol dialect dispatch of a function reference: the dialect gives each its own type, so this is a
/// parameter of the reference type rather than a property of one type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum FunctionReferenceKind {
    /// `mlir::sol::FuncRefType`, naming a module symbol, called by `sol.icall`.
    Internal = 0,
    /// `mlir::sol::ExtFuncRefType`, carrying an address and a selector, called by `sol.ext_icall`.
    External = 1,
}

impl From<slang_solidity_v2::ast::FunctionTypeVisibility> for FunctionReferenceKind {
    /// Only `external` dispatches through an address and selector; a bare reference to a `public`
    /// function is a legal internal pointer, as are `internal` and `private` ones.
    fn from(visibility: slang_solidity_v2::ast::FunctionTypeVisibility) -> Self {
        use slang_solidity_v2::ast::FunctionTypeVisibility as Slang;
        match visibility {
            Slang::External => Self::External,
            Slang::Public | Slang::Internal | Slang::Private => Self::Internal,
        }
    }
}
