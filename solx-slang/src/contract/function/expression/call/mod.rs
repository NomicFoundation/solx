//!
//! Function call emission: the one construct whose lowering is resolution-directed rather than
//! syntax-directed, dispatched by [`self::call_kind::CallKind`].
//!

pub mod arguments_declaration;
pub mod call_kind;
pub mod positional_arguments;

use self::call_kind::CallKind;

codegen!(
    /// A function call, dispatched by its [`CallKind`].
    FunctionCallExpression -> Values |node, scope| {
        CallKind::from_call(node).emit(node, scope)
    }
);
