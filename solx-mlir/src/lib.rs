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
pub mod ffi;

pub use self::attributes::contract_kind::ContractKind;
pub use self::attributes::icmp_predicate::ICmpPredicate;
pub use self::attributes::state_mutability::StateMutability;
pub use self::context::Context;
pub use self::context::builder::Builder;
pub use self::context::environment::Environment;
pub use self::context::environment::loop_target::LoopTarget;
