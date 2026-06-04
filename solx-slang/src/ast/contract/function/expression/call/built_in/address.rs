//!
//! Address value-transfer member built-ins: `address.send(amount)` and
//! `address.transfer(amount)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `address.send(amount)` to `sol.send`.
    pub fn emit_address_send(
        &self,
        _access: &MemberAccessExpression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("address.send")
    }

    /// Lowers `address.transfer(amount)` to `sol.transfer`.
    pub fn emit_address_transfer(
        &self,
        _access: &MemberAccessExpression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("address.transfer")
    }
}
