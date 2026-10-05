//!
//! The emission scope strata. A source unit scope owns the MLIR context, a contract scope borrows
//! it to hold the enclosing contract's state-variable data and body, a function scope borrows that
//! to hold a body's frame, and an assembly scope borrows that to hold an inline-assembly block's
//! Yul bindings. Each scope entity lives in its module here; its lowering methods live in the module
//! of the node they lower.
//!
//! A compilation unit scope, above them, holds the settings every definition of the unit is lowered
//! under, and lowers each object it deploys into a source unit scope of its own per segment.
//!

pub mod assembly;
pub mod compilation_unit;
pub mod contract;
pub mod function;
pub mod source_unit;
