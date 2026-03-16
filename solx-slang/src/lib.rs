//!
//! Slang Solidity frontend for solx.
//!

/// Slang AST construction from parsed compilation units.
pub mod ast;
/// Slang Solidity frontend implementation.
pub mod slang;

pub use self::ast::SemanticAst;
pub use self::slang::SlangFrontend;
pub use self::slang::compilation_config::SlangCompilationConfig;

#[cfg(test)]
mod tests;
