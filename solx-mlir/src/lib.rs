//!
//! MLIR integration for solx via melior.
//!
//! Provides low-level MLIR building primitives and LLVM translation
//! infrastructure. Frontend crates (e.g. `solx-slang`) use [`Context`]
//! to emit LLVM dialect operations without dealing with raw `melior` API
//! details, analogous to how `solx-yul` uses `solx-codegen-evm`.
//!

pub mod attributes;
pub mod context;
pub(crate) mod ffi;

pub use self::attributes::ContractKind;
pub use self::attributes::ICmpPredicate;
pub use self::attributes::StateMutability;
pub use self::context::Context;
pub use self::context::builder::Builder;
pub use self::context::environment::Environment;
pub use self::context::environment::loop_target::LoopTarget;
