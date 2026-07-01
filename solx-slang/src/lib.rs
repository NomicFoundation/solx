//!
//! Slang Solidity frontend for solx.
//!

#[macro_use]
extern crate solx_mlir;

/// Slang AST lowering to MLIR.
pub mod ast;
/// Slang Solidity frontend implementation.
pub mod slang;

pub use self::slang::Slang;
pub use self::slang::compilation_config::CompilationConfig;
