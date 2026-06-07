//!
//! Solidity built-in function and EVM intrinsic lowering.
//!

pub mod abi;
pub mod array;
pub mod global;
pub mod member_reference;
pub mod require;
pub mod type_introspection;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::Keccak256Operation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::Sha256Operation;

use crate::ast::contract::function::expression::call::CallEmitter;

/// ABI encoding mode for `abi.encode` / `abi.encodePacked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    /// Standard ABI encoding with per-element padding (`abi.encode`,
    /// `abi.encodeWithSelector`, `abi.encodeWithSignature`).
    Standard,
    /// Packed ABI encoding with no per-element padding (`abi.encodePacked`).
    Packed,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits an identifier-callee built-in (`assert`, `require`, `keccak256`,
    /// `sha256`, `ripemd160`, `ecrecover`, `addmod`, `mulmod`, `gasleft`).
    ///
    /// The returned value is `Some(...)` for value-producing built-ins and
    /// `None` for statement-style ones (`assert`, `require`). The caller
    /// ([`CallEmitter::classify_call`]) routes only handled built-ins with a
    /// matching argument count here, so the argument-count expectations always
    /// hold and unhandled variants are unreachable.
    ///
    /// # Errors
    ///
    /// Returns an error if the built-in's arguments are malformed (e.g. a
    /// non-string `require` message).
    pub fn emit_built_in_call(
        &self,
        built_in: BuiltIn,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        match built_in {
            BuiltIn::Assert => {
                let condition = arguments.iter().next().expect("assert has one argument");
                Ok((None, self.emit_assert(&condition, block)?))
            }
            BuiltIn::Require => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("require has a condition argument");
                let message = iter.next();
                Ok((
                    None,
                    self.emit_require(&condition, message.as_ref(), block)?,
                ))
            }
            BuiltIn::Gasleft => {
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        GasLeftOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("gasleft always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Keccak256 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        Keccak256Operation::builder(builder.context, builder.unknown_location)
                            .addr(values[0])
                            .result(builder.types.fixed_bytes(32))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("keccak256 always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Sha256 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        Sha256Operation::builder(builder.context, builder.unknown_location)
                            .data(values[0])
                            .result(builder.types.fixed_bytes(32))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sha256 always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Ripemd160 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        Ripemd160Operation::builder(builder.context, builder.unknown_location)
                            .data(values[0])
                            .result(builder.types.fixed_bytes(20))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("ripemd160 always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Ecrecover => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        EcrecoverOperation::builder(builder.context, builder.unknown_location)
                            .hash(values[0])
                            .v(values[1])
                            .r(values[2])
                            .s(values[3])
                            .result(builder.types.sol_address)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("ecrecover always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Addmod => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        AddModOperation::builder(builder.context, builder.unknown_location)
                            .x(values[0])
                            .y(values[1])
                            .r#mod(values[2])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("addmod always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Mulmod => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let value = block
                    .append_operation(
                        MulModOperation::builder(builder.context, builder.unknown_location)
                            .x(values[0])
                            .y(values[1])
                            .r#mod(values[2])
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("mulmod always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            _ => unreachable!("classify_call only routes emittable identifier built-ins here"),
        }
    }

    /// Emits a member-access built-in (e.g. `tx.origin`, `msg.sender`,
    /// `address.balance`, `abi.encode(...)`, `arr.push(...)`).
    ///
    /// Dispatches the resolved member built-in to its family handler; an
    /// unrecognized member is reported by [`Self::emit_environment_global`].
    ///
    /// # Errors
    ///
    /// Returns an error if the member access does not resolve to a recognized
    /// EVM intrinsic.
    pub fn emit_built_in_member_access(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // An enum-variant reference (`E.Variant`) resolves to a value, not a
        // built-in or intrinsic; handle it before the built-in dispatch.
        if let Some(ordinal) = self.enum_variant_ordinal(access, arguments) {
            let (value, block) = self.emit_enum_variant(access, ordinal, block);
            return Ok((Some(value), block));
        }
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => self.emit_address_balance(access, block),
            Some(BuiltIn::AddressCodehash) => self.emit_address_codehash(access, block),
            Some(BuiltIn::AddressCode) => self.emit_address_code(access, block),
            Some(BuiltIn::Length) => self.emit_member_length(access, block),
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
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, block)
            }
            Some(BuiltIn::TypeMin | BuiltIn::TypeMax) => {
                let (value, block) = self.emit_type_min_max(access, block)?;
                Ok((Some(value), block))
            }
            Some(BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax) => {
                let (value, block) = self.emit_type_enum_min_max(access, block)?;
                Ok((Some(value), block))
            }
            Some(BuiltIn::TypeInterfaceId) => {
                let (value, block) = self.emit_type_interface_id(access, block)?;
                Ok((Some(value), block))
            }
            Some(BuiltIn::TypeName) => {
                let (value, block) = self.emit_type_name(access, block)?;
                Ok((Some(value), block))
            }
            Some(BuiltIn::StringConcat | BuiltIn::BytesConcat) => {
                let arguments = arguments.expect("concat is a member-access call");
                self.emit_concat(arguments, block)
            }
            resolved => self.emit_environment_global(resolved, access, block),
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
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let builder = &self.expression_emitter.state.builder;
        let result_type = builder.types.string(solx_utils::DataLocation::Memory);
        let value = block
            .append_operation(
                ConcatOperation::builder(builder.context, builder.unknown_location)
                    .args(&values)
                    .result(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.concat always produces one result")
            .into();
        Ok((Some(value), block))
    }

    /// Emits each positional argument and returns the resulting values
    /// alongside the current block. The shared evaluation primitive that
    /// [`CallEmitter::emit_coerced_arguments`] builds on; `pub` so call sites in
    /// sibling modules (external/library/struct-constructor calls) reuse it
    /// rather than re-implementing the evaluation loop.
    pub fn emit_argument_values(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut values = Vec::with_capacity(arguments.len());
        let mut current = block;
        for argument in arguments.iter() {
            let (value, next) = self.expression_emitter.emit_value(&argument, current)?;
            values.push(value);
            current = next;
        }
        Ok((values, current))
    }

    /// Emits an intrinsic whose single operand is the receiver of a member
    /// access — e.g. `address.balance` (`sol.balance`), `address.codehash`
    /// (`sol.code_hash`), or `array.length` (`sol.length`).
    ///
    /// Evaluates the receiver, builds the operation via `build_op`, and
    /// extracts its single result.
    fn emit_unary_member_intrinsic<F>(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
        build_op: F,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>
    where
        F: FnOnce(Value<'context, 'block>) -> Operation<'context>,
    {
        let (address_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let value = block
            .append_operation(build_op(address_value))
            .result(0)
            .expect("unary member intrinsic always produces one result")
            .into();
        Ok((Some(value), block))
    }
}
