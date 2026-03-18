//!
//! MLIR integration for solx via melior.
//!
//! Provides low-level MLIR building primitives and LLVM translation
//! infrastructure. Frontend crates (e.g. `solx-slang`) use [`Context`]
//! to emit LLVM dialect operations without dealing with raw `melior` API
//! details, analogous to how `solx-yul` uses `solx-codegen-evm`.
//!

pub mod attributes;
pub mod builder;
pub mod environment;
pub mod ffi;
pub mod function_entry;
pub mod llvm_module;
pub mod ops;

pub use self::attributes::ContractKind;
pub use self::attributes::EvmVersion;
pub use self::attributes::ICmpPredicate;
pub use self::attributes::StateMutability;
pub use self::builder::Context;
pub use self::environment::Environment;
pub use self::environment::LoopTarget;
pub use self::function_entry::FunctionEntry;
pub use self::llvm_module::LlvmModule;

#[cfg(test)]
mod tests;
