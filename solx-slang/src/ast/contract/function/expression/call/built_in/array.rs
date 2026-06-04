//!
//! Dynamic-array and `bytes` member built-ins: `arr.push(x)`, `arr.push()`,
//! and `arr.pop()`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `arr.pop()` to `sol.pop`.
    pub fn emit_array_pop(
        &self,
        _access: &MemberAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("array.pop")
    }

    /// Lowers `arr.push(x)` / `arr.push()` to a storage push.
    pub fn emit_array_push(
        &self,
        _access: &MemberAccessExpression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("array.push")
    }
}
