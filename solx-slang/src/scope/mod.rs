//!
//! The emission scope strata. A source unit scope owns the MLIR context, a contract scope borrows
//! it to hold the enclosing contract's state-variable data and body, and a function scope borrows
//! that to hold a body's frame. Each scope entity lives in its module here; its lowering methods
//! live in the module of the node they lower.
//!

pub mod contract;
pub mod function;
pub mod source_unit;
