//!
//! The visibility of a function in the debug symbol table.
//!

use slang_solidity_v2::ast::FunctionTypeVisibility;
use slang_solidity_v2::ast::FunctionVisibility;

///
/// The visibility of a function.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// `public`.
    Public,
    /// `external`.
    External,
    /// `internal`.
    Internal,
    /// `private`.
    Private,
}

impl From<FunctionVisibility> for Visibility {
    fn from(visibility: FunctionVisibility) -> Self {
        match visibility {
            FunctionVisibility::Public => Self::Public,
            FunctionVisibility::External => Self::External,
            FunctionVisibility::Internal => Self::Internal,
            FunctionVisibility::Private => Self::Private,
        }
    }
}

impl From<FunctionTypeVisibility> for Visibility {
    fn from(visibility: FunctionTypeVisibility) -> Self {
        match visibility {
            FunctionTypeVisibility::Public => Self::Public,
            FunctionTypeVisibility::External => Self::External,
            FunctionTypeVisibility::Internal => Self::Internal,
            FunctionTypeVisibility::Private => Self::Private,
        }
    }
}
