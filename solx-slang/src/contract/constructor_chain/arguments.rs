//!
//! The argument list a base constructor of the chain receives.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;

/// The argument list a base constructor receives.
#[derive(Clone)]
pub struct Arguments {
    /// The chain position of the constructor providing the list, whose parameters it may name;
    /// absent when the providing contract declares none, leaving the list naming no parameter.
    pub provider: Option<usize>,
    /// The list as written, in the providing contract's inheritance specifier or in its
    /// constructor's modifier invocation.
    pub list: ArgumentsDeclaration,
}
