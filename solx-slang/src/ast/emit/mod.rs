//!
//! Slang AST emission traits: each node emits its own MLIR through a per-family trait implemented
//! directly on the Slang AST type (one trait per emission mode, threading the scope and current block).
//!

pub mod emit_as;
pub mod emit_constructor;
pub mod emit_expression;
pub mod emit_for_effect;
pub mod emit_function;
pub mod emit_modifier_calls;
pub mod emit_object;
pub mod emit_place;
pub mod emit_statement;
pub mod emit_values;
pub mod emit_yul;
