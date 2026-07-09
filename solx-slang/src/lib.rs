//!
//! Slang Solidity frontend for solx.
//!

pub(crate) mod ast;
pub(crate) mod slang;

pub use self::slang::Slang;
pub use self::slang::compilation_config::CompilationConfig;
