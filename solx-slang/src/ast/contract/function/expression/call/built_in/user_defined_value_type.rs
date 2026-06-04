//!
//! User-defined value type member built-ins: `T.wrap(x)` and `T.unwrap(x)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `T.wrap(x)` / `T.unwrap(x)` as a bit-level identity.
    pub fn emit_wrap_unwrap(
        &self,
        _call: &FunctionCallExpression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("UDVT wrap/unwrap")
    }
}
