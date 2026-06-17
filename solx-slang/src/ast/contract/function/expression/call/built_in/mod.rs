//!
//! Solidity built-in function and EVM intrinsic emission.
//!

use crate::ast::Type as AstType;
pub mod abi;
pub mod array;
pub mod global;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::ConcatOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

/// ABI encoding mode for `abi.encode` / `abi.encodePacked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    /// Standard ABI encoding with per-element padding (`abi.encode`,
    /// `abi.encodeWithSelector`, `abi.encodeWithSignature`).
    Standard,
    /// Packed ABI encoding with no per-element padding (`abi.encodePacked`).
    Packed,
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a call-position member-access built-in (`abi.encode(...)`,
    /// `arr.push(...)`, `addr.send(...)`, `string.concat(...)`).
    ///
    /// Dispatches the resolved member built-in to its family handler; an
    /// unrecognized member is a loud `unimplemented!`.
    pub fn emit_built_in_member_access(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressSend) => {
                let arguments = arguments.expect("send is a member-access call");
                self.emit_address_send(access, arguments, block)
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                self.emit_address_transfer(access, arguments, block)
            }
            Some(BuiltIn::AbiEncode) => {
                let arguments = arguments.expect("abi.encode is a member-access call");
                self.emit_abi_encode(arguments, block)
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let arguments = arguments.expect("abi.encodePacked is a member-access call");
                self.emit_abi_encode_packed(arguments, block)
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let arguments = arguments.expect("abi.encodeWithSelector is a member-access call");
                self.emit_abi_encode_with_selector(arguments, block)
            }
            Some(BuiltIn::AbiEncodeWithSignature) => {
                let arguments = arguments.expect("abi.encodeWithSignature is a member-access call");
                self.emit_abi_encode_with_signature(arguments, block)
            }
            Some(BuiltIn::AbiEncodeCall) => {
                let arguments = arguments.expect("abi.encodeCall is a member-access call");
                self.emit_abi_encode_call(arguments, block)
            }
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, block)
            }
            Some(BuiltIn::StringConcat | BuiltIn::BytesConcat) => {
                let arguments = arguments.expect("concat is a member-access call");
                self.emit_concat(arguments, block)
            }
            _ => unimplemented!(
                "unsupported call-position member built-in: {}",
                access.member().name()
            ),
        }
    }

    /// Lowers `string.concat(...)` / `bytes.concat(...)` to `sol.concat`, which
    /// takes a variadic list of string / `bytesN` values and yields a freshly
    /// allocated memory string. An empty argument list is valid
    /// (`string.concat()` → `""`).
    fn emit_concat(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let BlockAnd {
            value: values,
            block,
        } = arguments.emit(self, block);
        let builder = &self.state.builder;
        let result_type =
            AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
        let value = sol_op!(
            builder,
            block,
            ConcatOperation.args(&values).result(result_type)
        );
        (Some(value), block)
    }
}
