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

impl From<slang_solidity_v2::ast::CallMode> for FunctionReferenceKind {
    /// A delegate-called library function is a value only within the expression calling it, so
    /// its reference is external and the call site delegates. A partially applied function is
    /// only ever a callee, which Slang enforces.
    fn from(call_mode: slang_solidity_v2::ast::CallMode) -> Self {
        use slang_solidity_v2::ast::CallMode as Slang;
        match call_mode {
            Slang::Internal { bound: false } => Self::Internal,
            Slang::External | Slang::Delegate { bound: false } => Self::External,
            Slang::Internal { bound: true }
            | Slang::ExternalWithCallOptions
            | Slang::Delegate { bound: true } => {
                unreachable!("slang admits a partially applied function only as a callee")
            }
        }
    }
}
