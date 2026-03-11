//!
//! MLIR code generation from the Slang FlatAst.
//!
//! AST-driven emitters that lower Solidity constructs to MLIR using the
//! builder primitives from `solx_mlir`.
//!

pub mod contract;
pub mod expression;
pub mod function;
pub mod selector;
pub mod source_unit;
pub mod statement;
pub mod types;

pub use solx_mlir::MlirContext;
