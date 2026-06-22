//!
//! Slang Solidity frontend for solx.
//!

#[macro_use]
extern crate solx_mlir;

#[macro_use]
mod macros;

pub mod ast;
pub mod slang;

pub use self::slang::Slang;
pub use self::slang::compilation_config::CompilationConfig;
