//!
//! ABI encoding/decoding member built-ins: `abi.encode`, `abi.encodePacked`,
//! `abi.encodeWithSelector`, `abi.encodeWithSignature`, and `abi.decode`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Lowers `abi.encode(args)` to `sol.encode`.
    pub fn emit_abi_encode(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("abi.encode")
    }

    /// Lowers `abi.encodePacked(args)` to a packed `sol.encode`.
    pub fn emit_abi_encode_packed(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("abi.encodePacked")
    }

    /// Lowers `abi.encodeWithSelector(selector, args)`.
    pub fn emit_abi_encode_with_selector(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("abi.encodeWithSelector")
    }

    /// Lowers `abi.encodeWithSignature(signature, args)`.
    pub fn emit_abi_encode_with_signature(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("abi.encodeWithSignature")
    }

    /// Lowers `abi.decode(data, (types))` to `sol.decode`.
    pub fn emit_abi_decode(
        &self,
        _call: &FunctionCallExpression,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("abi.decode")
    }
}
