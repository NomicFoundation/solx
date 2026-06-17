//!
//! Slang Solidity frontend for solx.
//!

/// The ODS op-construction macros (`mlir_op!` / `mlir_op_build!` / `mlir_op_void!`)
/// live with the Builder in `solx-mlir`; pull them in crate-wide.
#[macro_use]
extern crate solx_mlir;

#[macro_use]
mod macros;

/// Slang AST emission to MLIR.
pub mod ast;
/// Slang Solidity frontend implementation.
pub mod slang;

pub use self::slang::Slang;
pub use self::slang::compilation_config::CompilationConfig;
