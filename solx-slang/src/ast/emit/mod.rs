//!
//! Slang AST emission traits: each node emits its own MLIR through a per-family
//! trait implemented directly on the Slang AST type.
//!
//! Each emission mode is one trait, threading the shared emission scope and the
//! current block: [`EmitExpression`] (a value), [`EmitStatement`] (a control-flow
//! continuation), [`EmitYul`] (inline assembly), [`EmitForEffect`] (an expression
//! in statement position), [`EmitPlace`] (an lvalue), and [`EmitAs`] (an expression
//! coerced to an expected type). The shared result types [`BlockAnd`](crate::ast::BlockAnd)
//! and [`Place`](crate::ast::Place) live beside this module.

pub mod emit_as;
pub mod emit_constructor;
pub mod emit_expression;
pub mod emit_for_effect;
pub mod emit_function;
pub mod emit_modifier_chain;
pub mod emit_object;
pub mod emit_place;
pub mod emit_statement;
pub mod emit_yul;

pub use self::emit_as::EmitAs;
pub use self::emit_constructor::EmitConstructor;
pub use self::emit_expression::EmitExpression;
pub use self::emit_for_effect::EmitForEffect;
pub use self::emit_function::EmitFunction;
pub use self::emit_modifier_chain::EmitModifierChain;
pub use self::emit_object::EmitObject;
pub use self::emit_place::EmitPlace;
pub use self::emit_statement::EmitStatement;
pub use self::emit_yul::EmitYul;
