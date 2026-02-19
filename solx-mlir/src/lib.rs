//!
//! MLIR integration for solx via melior.
//!
//! Provides an MLIR `Context` with all dialects and LLVM translations
//! registered, and binary MLIR-to-LLVM module translation (no text
//! serialization).
//!

pub mod context;
pub mod llvm_module;

pub use self::context::Context;
pub use self::llvm_module::LlvmModule;
