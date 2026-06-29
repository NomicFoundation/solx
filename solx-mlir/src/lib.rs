//!
//! MLIR integration for solx via melior.
//!
//! Provides MLIR building primitives for the Sol dialect and LLVM translation, used by frontend
//! crates (e.g. `solx-slang`) to emit Sol dialect operations without the raw `melior` API.
//!

#[macro_use]
mod macros;

pub mod attributes;
pub mod context;
pub mod dialect;
pub mod ffi;
pub mod ir;
pub mod llvm_module;
pub mod ods;
pub mod output;

pub use self::attributes::CmpPredicate;
pub use self::attributes::ContractKind;
pub use self::attributes::FunctionKind;
pub use self::attributes::StateMutability;
pub use self::attributes::YulCmpPredicate;
pub use self::context::Context;
pub use self::context::UserDefinedOperator;
pub use self::context::environment::Environment;
pub use self::context::function::Function;
pub use self::context::modifier::Modifier;
pub use self::context::try_fallback_kind::TryFallbackKind;
pub use self::dialect::Dialect;
pub use self::ir::pointer::Pointer;
pub use self::ir::r#type::Type;
pub use self::ir::r#type::array_size::ArraySize;
pub use self::ir::r#type::location_policy::LocationPolicy;
pub use self::ir::value::Value;
pub use self::ir::yul_value::YulValue;
pub use self::macros::IntoOds;
pub use self::output::MlirOutput;
