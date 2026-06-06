//!
//! `new` expression lowering: dynamic-aggregate allocation (`new T[](n)`,
//! `new bytes(n)`, `new string(n)`) and contract creation (`new C(args)`).
//!
//! An [`ExpressionEmitter`] method: `new.rs` lives in the expression module
//! subtree, so it lowers through the expression emitter directly rather than
//! the call emitter (the oracle's `built_in/new.rs` placement).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `new` expression: dynamic-aggregate allocation (`new T[](n)`,
    /// `new bytes(n)`) or contract creation (`new C(args)`).
    pub fn emit_new(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (call, arguments, block);
        unimplemented!("new expression")
    }
}
