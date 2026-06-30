//!
//! MLIR integration for solx via melior.
//!
//! Provides MLIR building primitives for the Sol dialect and LLVM translation, used by frontend
//! crates (e.g. `solx-slang`) to emit Sol dialect operations without the raw `melior` API.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]
#![allow(clippy::new_without_default)]

#[macro_use]
mod macros;

pub mod context;
pub mod dialect;
pub(crate) mod ffi;
pub mod ir;
pub mod llvm_module;
pub mod ods;
pub mod output;

pub use self::context::Context;
pub use self::context::environment::Environment;
pub use self::context::function::Function;
pub use self::context::modifier::Modifier;
pub use self::context::user_defined_operator::UserDefinedOperator;
pub use self::dialect::Dialect;
pub use self::ir::attributes::CmpPredicate;
pub use self::ir::attributes::ContractKind;
pub use self::ir::attributes::FunctionKind;
pub use self::ir::attributes::StateMutability;
pub use self::ir::attributes::YulCmpPredicate;
pub use self::ir::pointer::Pointer;
pub use self::ir::r#type::Type;
pub use self::ir::r#type::array_size::ArraySize;
pub use self::ir::r#type::location_policy::LocationPolicy;
pub use self::ir::value::Value;
pub use self::ir::yul_value::YulValue;
pub use self::macros::IntoOds;
pub use self::output::MlirOutput;
