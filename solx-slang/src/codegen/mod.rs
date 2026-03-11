//!
//! MLIR code generation from the Slang FlatAst.
//!
//! AST-driven emitters that lower Solidity constructs to MLIR using the
//! builder primitives from `solx_mlir`.
//!

/// Contract definition lowering to MLIR.
pub(crate) mod contract;
/// Expression lowering to MLIR SSA values.
pub(crate) mod expression;
/// Function definition lowering to MLIR.
pub(crate) mod function;
/// EVM function selector computation.
pub(crate) mod selector;
/// Source unit (top-level file) lowering to MLIR.
pub(crate) mod source_unit;
/// Statement lowering to MLIR operations.
pub(crate) mod statement;
/// Solidity to MLIR type mapping.
pub(crate) mod types;

pub use solx_mlir::MlirContext;
