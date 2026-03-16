//!
//! MLIR integration for solx via melior.
//!
//! Provides low-level MLIR building primitives and LLVM translation
//! infrastructure. Frontend crates (e.g. `solx-slang`) use [`MlirContext`]
//! to emit LLVM dialect operations without dealing with raw `melior` API
//! details, analogous to how `solx-yul` uses `solx-codegen-evm`.
//!

pub mod builder;
pub mod context;
pub mod environment;
pub mod ffi;
pub mod function_entry;
pub mod icmp_predicate;
pub mod llvm_module;
pub mod loop_target;
pub mod ops;

pub use self::builder::MlirContext;
pub use self::context::Context;
pub use self::environment::Environment;
pub use self::function_entry::FunctionEntry;
pub use self::icmp_predicate::ICmpPredicate;
pub use self::llvm_module::LlvmModule;
pub use self::loop_target::LoopTarget;

#[cfg(test)]
mod tests;
