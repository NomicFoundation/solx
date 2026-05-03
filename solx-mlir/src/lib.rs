//!
//! MLIR integration for solx via melior.
//!
//! Provides MLIR building primitives for the Sol dialect and LLVM translation
//! infrastructure. Frontend crates (e.g. `solx-slang`) use [`Context`]
//! to emit Sol dialect operations without dealing with raw `melior` API
//! details, analogous to how `solx-yul` uses `solx-codegen-evm`.
//!

#![expect(
    clippy::too_many_arguments,
    reason = "MLIR builder methods require many parameters for operation construction"
)]

pub mod attributes;
pub mod context;
pub mod dialect;
pub mod ffi;
pub mod llvm_module;
pub mod ods;

pub use self::attributes::cmp_predicate::CmpPredicate;
pub use self::attributes::contract_kind::ContractKind;
pub use self::attributes::function_kind::FunctionKind;
pub use self::attributes::state_mutability::StateMutability;
pub use self::context::Context;
pub use self::context::builder::Builder;
pub use self::context::builder::type_factory::TypeFactory;
pub use self::context::builder::type_factory::array_size::ArraySize;
pub use self::context::environment::Environment;
pub use self::dialect::Dialect;
