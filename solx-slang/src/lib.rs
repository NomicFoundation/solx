//!
//! Slang Solidity frontend for solx: the Slang AST lowered to Sol dialect MLIR, laid out as the
//! tree the language nests: the source unit's module scope holds contracts, a contract holds its
//! functions and state variables, a function holds statements, and statements evaluate expressions.
//! Each node's lowering lives in the file named for it at its nesting depth.
//!

pub(crate) mod contract;
pub(crate) mod scope;
pub(crate) mod slang;
pub(crate) mod source_unit;
pub(crate) mod r#type;

pub use self::slang::Slang;
