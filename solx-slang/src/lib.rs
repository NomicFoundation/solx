//!
//! Slang Solidity frontend for solx.
//!

/// Slang AST lowering to MLIR.
pub mod ast;
/// Slang Solidity frontend implementation.
pub mod slang;

pub use self::slang::SlangFrontend;
pub use self::slang::compilation_config::CompilationConfig;

#[cfg(test)]
mod tests;
