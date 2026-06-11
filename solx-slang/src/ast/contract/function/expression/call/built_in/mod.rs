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
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::BlockHashOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::Keccak256Operation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::Sha256Operation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::type_conversion::TypeConversion;

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
            BuiltIn::Blockhash => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                // `sol.blockhash` takes a `ui256` block number; coerce a narrower
                // argument type up first.
                let block_number = TypeConversion::from_target_type(builder.types.ui256, builder)
                    .emit(values[0], builder, &block);
                let value = block
                    .append_operation(
                        BlockHashOperation::builder(builder.context, builder.unknown_location)
                            .block_number(block_number)
                            .val(builder.types.fixed_bytes(32))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("blockhash always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            BuiltIn::Keccak256 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                // `sol.keccak256` hashes a memory buffer; a storage / calldata
                // `bytes` argument is a reference, so copy it to memory first
                // (solc emits a Storage|CallData -> Memory `sol.data_loc_cast`
                // here). An already-memory buffer passes through unchanged.
                let input =
                    TypeConversion::from_target_type(builder.types.sol_string_memory, builder)
                        .emit(values[0], builder, &block);
                let value = block
                    .append_operation(
                        Keccak256Operation::builder(builder.context, builder.unknown_location)
                            .addr(input)
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
                // `addmod` operates on `uint256`, but a literal operand keeps its
                // narrow type (`addmod(1, 2, d)` → ui8, ui8, ui256); `sol.addmod`
                // requires identical operand/result types, so widen all to ui256.
                let ui256 = builder.types.ui256;
                let x = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[0], builder, &block);
                let y = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[1], builder, &block);
                let modulus = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[2], builder, &block);
                let value = block
                    .append_operation(
                        AddModOperation::builder(builder.context, builder.unknown_location)
                            .x(x)
                            .y(y)
                            .r#mod(modulus)
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
                // `mulmod` operates on `uint256`; widen narrow literal operands so
                // all operands/result share the type `sol.mulmod` requires.
                let ui256 = builder.types.ui256;
                let x = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[0], builder, &block);
                let y = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[1], builder, &block);
                let modulus = TypeConversion::from_target_type(ui256, builder)
                    .emit(values[2], builder, &block);
                let value = block
                    .append_operation(
                        MulModOperation::builder(builder.context, builder.unknown_location)
                            .x(x)
                            .y(y)
                            .r#mod(modulus)
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
        // A function-like built-in member over a value operand referenced
        // WITHOUT a call — `addr.transfer` / `addr.send` / `addr.call` /
        // `delegatecall` / `staticcall`, or `data.pop` / `data.push` — e.g. a
        // discarded `payable(this).transfer;` or `data.pop;` statement — is a
        // member reference, not the action itself (which the call dispatch
        // handles; for the uncalled form solc only binds the function and checks
        // its stack size). Evaluate the operand for its side effects and yield a
        // placeholder value. (The `abi.*` forms are also no-ops uncalled but
        // their operand is the `abi` namespace keyword, not a value, so they are
        // not bound here.)
        if arguments.is_none()
            && matches!(
                access.member().resolve_to_built_in(),
                Some(
                    BuiltIn::AddressTransfer
                        | BuiltIn::AddressSend
                        | BuiltIn::AddressCall
                        | BuiltIn::AddressDelegatecall
                        | BuiltIn::AddressStaticcall
                        | BuiltIn::ArrayPop
                        | BuiltIn::ArrayPush
                )
            )
        {
            let (_operand, block) = self
                .expression_emitter
                .emit_value(&access.operand(), block)?;
            let builder = &self.expression_emitter.state.builder;
            let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
            return Ok((Some(placeholder), block));
        }
        // The `abi.*` builtins referenced WITHOUT a call — `abi.encode;`,
        // `abi.encodePacked;`, `abi.decode;` and friends, e.g. a discarded
        // statement — are no-ops too, but unlike the actions above their operand
        // is the `abi` namespace keyword, not a value, so nothing is evaluated
        // (binding `abi` would itself fail); just yield a placeholder.
        if arguments.is_none()
            && matches!(
                access.member().resolve_to_built_in(),
                Some(
                    BuiltIn::AbiEncode
                        | BuiltIn::AbiEncodePacked
                        | BuiltIn::AbiEncodeWithSelector
                        | BuiltIn::AbiEncodeWithSignature
                        | BuiltIn::AbiEncodeCall
                        | BuiltIn::AbiDecode
                )
            )
        {
            let builder = &self.expression_emitter.state.builder;
            let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
            return Ok((Some(placeholder), block));
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
            Some(BuiltIn::AbiEncodeCall) => {
                let arguments = arguments.expect("abi.encodeCall is a member-access call");
                self.emit_abi_encode_call(arguments, block)
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
            Some(BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode) => {
                let (value, block) = self.emit_type_code(access, block)?;
                Ok((Some(value), block))
            }
            Some(BuiltIn::StringConcat | BuiltIn::BytesConcat) => {
                let arguments = arguments.expect("concat is a member-access call");
                self.emit_concat(arguments, block)
            }
            Some(BuiltIn::FunctionSelector) => self.emit_function_selector(access, block),
            Some(BuiltIn::FunctionAddress) => self.emit_function_address(access, block),
            Some(BuiltIn::ErrorSelector) => self.emit_error_selector(access, block),
            Some(BuiltIn::EventSelector) => self.emit_event_selector(access, block),
            // A bare `T.wrap` / `T.unwrap` named without a call (a discarded
            // `(MyInt).wrap;` statement) references the built-in itself, which
            // has no runtime value. The call forms `T.wrap(x)` / `T.unwrap(v)`
            // lower in the call dispatch (`CallKind::UdvtWrapUnwrap`); a bare
            // reference is a no-op, so yield a placeholder.
            Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                let builder = &self.expression_emitter.state.builder;
                let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                Ok((Some(placeholder), block))
            }
            // A member that resolves to a function used as a value (not called) is
            // a function pointer: an externally-visible function with a selector
            // (`this.f`, `instance.f`) is an external pointer, while a
            // namespace-qualified internal function with none (`C.f`, `(L.f)`) is
            // an internal pointer (`sol.func_constant`), like a bare `f`.
            resolved => {
                if let Some(Definition::Function(function_definition)) =
                    access.member().resolve_to_definition()
                {
                    if function_definition.compute_selector().is_some() {
                        self.emit_external_function_pointer(access, &function_definition, block)
                    } else {
                        // The literal target lowers (no virtual redirect): an
                        // explicit `Base.f` names Base's own implementation, not
                        // the most-derived override a bare `f` would bind.
                        let (mlir_name, parameter_types, return_types) = self
                            .expression_emitter
                            .state
                            .resolve_function(function_definition.node_id())?;
                        let func_ref_type = self
                            .expression_emitter
                            .state
                            .builder
                            .types
                            .func_ref(parameter_types, return_types);
                        let mlir_name = mlir_name.to_owned();
                        let value = self
                            .expression_emitter
                            .state
                            .builder
                            .emit_sol_func_constant(&mlir_name, func_ref_type, &block);
                        Ok((Some(value), block))
                    }
                } else {
                    self.emit_environment_global(resolved, access, block)
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
