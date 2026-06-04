//!
//! Built-in function call lowering.
//!

/// ABI encoding/decoding member built-ins (`abi.encode*` / `abi.decode`).
pub mod abi;
/// Address value-transfer member built-ins (`send`/`transfer`).
pub mod address;
/// Dynamic-array and `bytes` member built-ins (`push`/`pop`).
pub mod array;
/// User-defined value type member built-ins (`wrap`/`unwrap`).
pub mod user_defined_value_type;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to lower `callee(arguments)` as a Solidity built-in.
    ///
    /// Returns `Ok(Some((value, block)))` on a recognized built-in — the value
    /// is `None` for statement-style built-ins (`assert`, `require`) — or
    /// `Ok(None)` when the callee is not a built-in, so the caller falls
    /// through. The binder fixes each built-in's arity, so the handlers take
    /// the arguments as given. Member-access built-ins (`msg.sender`,
    /// `abi.encode`, …) and the remaining globals defer to later domains.
    pub fn try_emit_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::Identifier(identifier) = callee else {
            return Ok(None);
        };
        let Some(built_in) = identifier.resolve_to_built_in() else {
            return Ok(None);
        };
        match built_in {
            BuiltIn::Assert => self.emit_assert(arguments, block).map(Some),
            BuiltIn::Require => self.emit_require(arguments, block).map(Some),
            BuiltIn::Gasleft => self.emit_gasleft(block).map(Some),
            BuiltIn::Keccak256 => self.emit_keccak256(arguments, block).map(Some),
            BuiltIn::Sha256 => self.emit_sha256(arguments, block).map(Some),
            BuiltIn::Ripemd160 => self.emit_ripemd160(arguments, block).map(Some),
            BuiltIn::Ecrecover => self.emit_ecrecover(arguments, block).map(Some),
            BuiltIn::Addmod => self.emit_addmod(arguments, block).map(Some),
            BuiltIn::Mulmod => self.emit_mulmod(arguments, block).map(Some),
            _ => Ok(None),
        }
    }

    /// Tries to lower a member-access call `base.method(args)` whose method is a
    /// Solidity built-in handled here: the array `push`/`pop`, the address
    /// value-transfer `send`/`transfer`, and the `abi.*` encode/decode family.
    ///
    /// Returns `Ok(None)` when the callee is not a member access or its member
    /// is not such a built-in, so the caller falls through to the remaining
    /// call kinds.
    pub fn try_emit_member_built_in_call(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::MemberAccessExpression(access) = call.operand() else {
            return Ok(None);
        };
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(&access, block).map(Some),
            Some(BuiltIn::ArrayPush) => self.emit_array_push(&access, arguments, block).map(Some),
            Some(BuiltIn::AddressSend) => {
                self.emit_address_send(&access, arguments, block).map(Some)
            }
            Some(BuiltIn::AddressTransfer) => self
                .emit_address_transfer(&access, arguments, block)
                .map(Some),
            Some(BuiltIn::AbiEncode) => self.emit_abi_encode(arguments, block).map(Some),
            Some(BuiltIn::AbiEncodePacked) => {
                self.emit_abi_encode_packed(arguments, block).map(Some)
            }
            Some(BuiltIn::AbiEncodeWithSelector) => self
                .emit_abi_encode_with_selector(arguments, block)
                .map(Some),
            Some(BuiltIn::AbiEncodeWithSignature) => self
                .emit_abi_encode_with_signature(arguments, block)
                .map(Some),
            Some(BuiltIn::AbiDecode) => self.emit_abi_decode(call, arguments, block).map(Some),
            Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                self.emit_wrap_unwrap(call, arguments, block).map(Some)
            }
            _ => Ok(None),
        }
    }

    /// Lowers `assert(condition)` to `sol.assert`.
    fn emit_assert(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: assert")
    }

    /// Lowers `require(condition[, message])` to `sol.require`.
    fn emit_require(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: require")
    }

    /// Lowers `gasleft()` to `sol.gasleft`.
    fn emit_gasleft(
        &self,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: gasleft")
    }

    /// Lowers `keccak256(bytes memory)` to `sol.keccak256`.
    fn emit_keccak256(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: keccak256")
    }

    /// Lowers `sha256(bytes memory)` to the `sol.sha256` precompile.
    fn emit_sha256(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: sha256")
    }

    /// Lowers `ripemd160(bytes memory)` to the `sol.ripemd160` precompile.
    fn emit_ripemd160(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: ripemd160")
    }

    /// Lowers `ecrecover(hash, v, r, s)` to the `sol.ecrecover` precompile.
    fn emit_ecrecover(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: ecrecover")
    }

    /// Lowers `addmod(x, y, m)` to `sol.addmod`.
    fn emit_addmod(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: addmod")
    }

    /// Lowers `mulmod(x, y, m)` to `sol.mulmod`.
    fn emit_mulmod(
        &self,
        _arguments: &PositionalArguments,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        unimplemented!("built-in: mulmod")
    }
}
