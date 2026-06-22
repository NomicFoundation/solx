//!
//! Slang AST emission to MLIR.
//!

pub mod block_and;
pub mod contract;
pub mod emit;
pub mod operator_binding;
pub mod pending_queries;
pub mod place;

pub use self::block_and::BlockAnd;
pub use self::emit::EmitAs;
pub use self::emit::EmitExpression;
pub use self::emit::EmitForEffect;
pub use self::emit::EmitFunction;
pub use self::emit::EmitObject;
pub use self::emit::EmitPlace;
pub use self::emit::EmitStatement;
pub use self::emit::EmitYul;
pub use self::place::Place;
pub use solx_mlir::LocationPolicy;
pub use solx_mlir::Pointer;
pub use solx_mlir::Type;
pub use solx_mlir::Value;
