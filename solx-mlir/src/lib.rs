//!
//! MLIR integration for solx via melior.
//!
//! Provides MLIR building primitives for the Sol dialect and LLVM translation, used by frontend
//! crates (e.g. `solx-slang`) to emit Sol dialect operations without the raw `melior` API.
//!

#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]

#[macro_use]
mod macros;

pub mod context;
pub mod dialect;
pub mod ffi;
pub mod ir;
pub mod llvm_module;
pub mod ods;
pub mod output;

pub use self::context::Context;
pub use self::context::environment::Environment;
pub use self::context::function::Function;
pub use self::context::user_defined_operator::UserDefinedOperator;
pub use self::dialect::Dialect;
pub use self::ir::attributes::CmpPredicate;
pub use self::ir::attributes::ContractKind;
pub use self::ir::attributes::FunctionKind;
pub use self::ir::attributes::StateMutability;
pub use self::ir::pointer::Pointer;
pub use self::ir::r#type::Type;
pub use self::ir::r#type::array_size::ArraySize;
pub use self::ir::value::Value;
pub use self::macros::IntoOds;
pub use self::output::MlirOutput;
