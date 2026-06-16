//!
//! Solidity built-in function and EVM intrinsic emission.
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
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::BlockHashOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::Keccak256Operation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::Sha256Operation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;

/// ABI encoding mode for `abi.encode` / `abi.encodePacked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    /// Standard ABI encoding with per-element padding (`abi.encode`,
    /// `abi.encodeWithSelector`, `abi.encodeWithSignature`).
    Standard,
    /// Packed ABI encoding with no per-element padding (`abi.encodePacked`).
    Packed,
}

impl CallKind {
    /// Emits an identifier-callee built-in (`assert`, `require`, `keccak256`,
    /// `sha256`, `ripemd160`, `ecrecover`, `addmod`, `mulmod`, `gasleft`).
    ///
    /// `Some(...)` for value-producing built-ins, `None` for statement-style
    /// ones (`assert`, `require`). Only handled built-ins with a matching
    /// argument count reach here, so the expectations hold.
    pub fn emit_built_in_call<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        built_in: BuiltIn,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        match built_in {
            BuiltIn::Assert => {
                let condition = arguments.iter().next().expect("assert has one argument");
                (None, context.emit_assert(&condition, block))
            }
            BuiltIn::Require => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("require has a condition argument");
                let message = iter.next();
                (
                    None,
                    context.emit_require(&condition, message.as_ref(), block),
                )
            }
            BuiltIn::Gasleft => (
                Some(crate::ast::Value::gas_left(&context.state.builder, &block).into_mlir()),
                block,
            ),
            BuiltIn::Blockhash => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                // `sol.blockhash` takes a `ui256` block number; coerce a narrower
                // argument type up first.
                let block_number = crate::ast::Value::from(values[0])
                    .cast(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                        builder,
                        &block,
                    )
                    .into_mlir();
                let value = sol_op!(
                    builder,
                    block,
                    BlockHashOperation
                        .block_number(block_number)
                        .val(crate::ast::Type::fixed_bytes(builder.context, 32).into_mlir())
                );
                (Some(value), block)
            }
            BuiltIn::Keccak256 => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let value = context.emit_keccak256(values[0], &block);
                (Some(value), block)
            }
            BuiltIn::Sha256 => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                let value = sol_op!(
                    builder,
                    block,
                    Sha256Operation
                        .data(values[0])
                        .result(crate::ast::Type::fixed_bytes(builder.context, 32).into_mlir())
                );
                (Some(value), block)
            }
            BuiltIn::Ripemd160 => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                let value = sol_op!(
                    builder,
                    block,
                    Ripemd160Operation
                        .data(values[0])
                        .result(crate::ast::Type::fixed_bytes(builder.context, 20).into_mlir())
                );
                (Some(value), block)
            }
            BuiltIn::Ecrecover => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                // `ecrecover(bytes32 hash, uint8 v, bytes32 r, bytes32 s)`: the
                // hash / r / s arguments keep their literal `uint256` type, but
                // `sol.ecrecover` takes `fixedbytes<32>` for them and `ui8` for
                // `v`. Coerce each to its signature type (matching solc).
                let bytes32 = crate::ast::Type::fixed_bytes(builder.context, 32).into_mlir();
                let ui8 = Type::from(IntegerType::unsigned(builder.context, 8));
                let hash = crate::ast::Value::from(values[0])
                    .cast(crate::ast::Type::new(bytes32), builder, &block)
                    .into_mlir();
                let v = crate::ast::Value::from(values[1])
                    .cast(crate::ast::Type::new(ui8), builder, &block)
                    .into_mlir();
                let r = crate::ast::Value::from(values[2])
                    .cast(crate::ast::Type::new(bytes32), builder, &block)
                    .into_mlir();
                let s = crate::ast::Value::from(values[3])
                    .cast(crate::ast::Type::new(bytes32), builder, &block)
                    .into_mlir();
                let value = sol_op!(
                    builder,
                    block,
                    EcrecoverOperation
                        .hash(hash)
                        .v(v)
                        .r(r)
                        .s(s)
                        .result(crate::ast::Type::address(builder.context, false).into_mlir())
                );
                (Some(value), block)
            }
            BuiltIn::Addmod => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                // `addmod` operates on `uint256`, but a literal operand keeps its
                // narrow type (`addmod(1, 2, d)` → ui8, ui8, ui256); `sol.addmod`
                // requires identical operand/result types, so widen all to ui256.
                let ui256 =
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir();
                let x = crate::ast::Value::from(values[0])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let y = crate::ast::Value::from(values[1])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let modulus = crate::ast::Value::from(values[2])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let value = sol_op!(builder, block, AddModOperation.x(x).y(y).r#mod(modulus));
                (Some(value), block)
            }
            BuiltIn::Mulmod => {
                let BlockAnd {
                    value: values,
                    block,
                } = arguments.emit(context, block);
                let builder = &context.state.builder;
                // `mulmod` operates on `uint256`; widen narrow literal operands so
                // all operands/result share the type `sol.mulmod` requires.
                let ui256 =
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir();
                let x = crate::ast::Value::from(values[0])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let y = crate::ast::Value::from(values[1])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let modulus = crate::ast::Value::from(values[2])
                    .cast(crate::ast::Type::new(ui256), builder, &block)
                    .into_mlir();
                let value = sol_op!(builder, block, MulModOperation.x(x).y(y).r#mod(modulus));
                (Some(value), block)
            }
            _ => unreachable!("only emittable identifier built-ins are routed here"),
        }
    }
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a member-access built-in (e.g. `tx.origin`, `msg.sender`,
    /// `address.balance`, `abi.encode(...)`, `arr.push(...)`).
    ///
    /// Dispatches the resolved member built-in to its family handler; an
    /// unrecognized member is reported by [`Self::emit_environment_global`].
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
        // A function-like built-in member referenced WITHOUT a call —
        // `addr.transfer`/`send`/`call`/`delegatecall`/`staticcall`,
        // `data.pop`/`push`, e.g. a discarded `data.pop;` — is a member
        // reference, not the action (which the call dispatch handles). solc
        // only binds the function; evaluate the operand for its side effects
        // and yield a placeholder. (`abi.*` is handled below: its operand is
        // the `abi` namespace keyword, not a value.)
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
            let BlockAnd {
                value: _operand,
                block,
            } = access.operand().emit(self, block);
            let builder = &self.state.builder;
            let placeholder = crate::ast::Value::constant(
                0,
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .into_mlir();
            return (Some(placeholder), block);
        }
        // The `abi.*` builtins referenced WITHOUT a call (e.g. a discarded
        // `abi.encode;`) are no-ops; their operand is the `abi` namespace
        // keyword, not a value, so nothing is evaluated (binding `abi` would
        // fail). Yield a placeholder.
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
            let builder = &self.state.builder;
            let placeholder = crate::ast::Value::constant(
                0,
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .into_mlir();
            return (Some(placeholder), block);
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
                let (value, block) = self.emit_type_min_max(access, block);
                (Some(value), block)
            }
            Some(BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax) => {
                let (value, block) = self.emit_type_enum_min_max(access, block);
                (Some(value), block)
            }
            Some(BuiltIn::TypeInterfaceId) => {
                let (value, block) = self.emit_type_interface_id(access, block);
                (Some(value), block)
            }
            Some(BuiltIn::TypeName) => {
                let (value, block) = self.emit_type_name(access, block);
                (Some(value), block)
            }
            Some(BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode) => {
                let (value, block) = self.emit_type_code(access, block);
                (Some(value), block)
            }
            Some(BuiltIn::StringConcat | BuiltIn::BytesConcat) => {
                let arguments = arguments.expect("concat is a member-access call");
                self.emit_concat(arguments, block)
            }
            Some(BuiltIn::FunctionSelector) => self.emit_function_selector(access, block),
            Some(BuiltIn::FunctionAddress) => self.emit_function_address(access, block),
            Some(BuiltIn::ErrorSelector) => self.emit_error_selector(access, block),
            Some(BuiltIn::EventSelector) => self.emit_event_selector(access, block),
            // `T.wrap` / `T.unwrap` named without a call is a no-op reference to
            // the built-in itself (the call forms lower in the call dispatch via
            // `CallKind::UdvtWrapUnwrap`). Yield a placeholder.
            Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                let builder = &self.state.builder;
                let placeholder = crate::ast::Value::constant(
                    0,
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                    builder,
                    &block,
                )
                .into_mlir();
                (Some(placeholder), block)
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
                        let (value, block) =
                            self.emit_function_constant(function_definition.node_id(), block);
                        (Some(value), block)
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
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let BlockAnd {
            value: values,
            block,
        } = arguments.emit(self, block);
        let builder = &self.state.builder;
        let result_type =
            crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
        let value = sol_op!(
            builder,
            block,
            ConcatOperation.args(&values).result(result_type)
        );
        (Some(value), block)
    }

    /// Emits `keccak256` over a byte buffer, returning the 32-byte hash. The
    /// buffer is coerced to memory first — a storage / calldata `bytes` is a
    /// reference, which solc copies to memory before hashing (`sol.keccak256`
    /// hashes a memory buffer) — a no-op when the buffer is already memory.
    /// Shared by the `keccak256` built-in and `abi.encodeWithSignature`'s
    /// runtime-signature hash.
    pub fn emit_keccak256(
        &self,
        buffer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        let input = crate::ast::Value::from(buffer)
            .cast(
                crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory),
                builder,
                block,
            )
            .into_mlir();
        sol_op!(
            builder,
            block,
            Keccak256Operation
                .addr(input)
                .result(crate::ast::Type::fixed_bytes(builder.context, 32).into_mlir())
        )
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
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>)
    where
        F: FnOnce(Value<'context, 'block>) -> Operation<'context>,
    {
        let BlockAnd {
            value: address_value,
            block,
        } = access.operand().emit(self, block);
        let value = block
            .append_operation(build_op(address_value.into_mlir()))
            .result(0)
            .expect("unary member intrinsic always produces one result")
            .into();
        (Some(value), block)
    }
}
