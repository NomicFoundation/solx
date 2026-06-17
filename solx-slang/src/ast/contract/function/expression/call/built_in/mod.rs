//!
//! Solidity built-in function and EVM intrinsic emission.
//!

use crate::ast::Type as AstType;
pub mod abi;
pub mod array;
pub mod global;
pub mod member_reference;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
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
    /// Emits a member-access built-in (e.g. `tx.origin`, `msg.sender`,
    /// `address.balance`, `abi.encode(...)`, `arr.push(...)`).
    ///
    /// Dispatches the resolved member built-in to its family handler; an
    /// unrecognized member is a loud `unimplemented!`.
    pub fn emit_built_in_member_access(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        // An enum-variant reference (`E.Variant`) resolves to a value, not a
        // built-in or intrinsic; handle it before the built-in dispatch.
        if let Some(ordinal) = self.enum_variant_ordinal(access, arguments) {
            let (value, block) = self.emit_enum_variant(access, ordinal, block);
            return (Some(value), block);
        }
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
            Some(BuiltIn::FunctionSelector) => self.emit_function_selector(access, block),
            Some(BuiltIn::FunctionAddress) => self.emit_function_address(access, block),
            Some(BuiltIn::ErrorSelector) => self.emit_error_selector(access, block),
            Some(BuiltIn::EventSelector) => self.emit_event_selector(access, block),
            // A member that resolves to a function used as a value (not called) is
            // a function pointer: an externally-visible function with a selector
            // (`this.f`, `instance.f`) is an external pointer, while a
            // namespace-qualified internal function with none (`C.f`, `(L.f)`) is
            // an internal pointer (`sol.func_constant`), like a bare `f`.
            _ => {
                let Some(Definition::Function(function_definition)) =
                    access.member().resolve_to_definition()
                else {
                    unimplemented!("unsupported member access: {}", access.member().name());
                };
                if function_definition.compute_selector().is_some() {
                    self.emit_external_function_pointer(access, &function_definition, block)
                } else {
                    // The literal target lowers (no virtual redirect): an explicit
                    // `Base.f` names Base's own implementation, not the most-derived
                    // override a bare `f` would bind.
                    let (value, block) =
                        self.emit_function_constant(function_definition.node_id(), block);
                    (Some(value), block)
                }
            }
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
