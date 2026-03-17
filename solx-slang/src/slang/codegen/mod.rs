//!
//! MLIR code generation from the Slang SemanticAst.
//!
//! AST-driven emitters that lower Solidity constructs to MLIR using the
//! builder primitives from `solx_mlir`.
//!

/// Source unit (top-level file) lowering to MLIR.
pub(crate) mod source_unit;
/// Solidity to MLIR type mapping.
pub(crate) mod types;

pub use solx_mlir::MlirContext;
