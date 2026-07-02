//!
//! Member-position Solidity built-in calls and EVM intrinsics.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::operation::OperationMutLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::BalanceOperation;
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_mlir::ods::sol::BaseFeeOperation;
use solx_mlir::ods::sol::BlobBaseFeeOperation;
use solx_mlir::ods::sol::BlockNumberOperation;
use solx_mlir::ods::sol::BytesCastOperation;
use solx_mlir::ods::sol::CallValueOperation;
use solx_mlir::ods::sol::CallerOperation;
use solx_mlir::ods::sol::ChainIdOperation;
use solx_mlir::ods::sol::CodeHashOperation;
use solx_mlir::ods::sol::CodeOperation;
use solx_mlir::ods::sol::CoinbaseOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::DifficultyOperation;
use solx_mlir::ods::sol::EncodeOperation;
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Tries to emit a built-in that needs the full [`FunctionCallExpression`]
    /// context — typically because the result type comes from `call.get_type()`
    /// rather than from the operands (e.g. `abi.decode(payload, (T))`).
    ///
    /// Resolves the callee's member access to a [`BuiltIn`] variant and
    /// dispatches to the matching handler. Returns `Some((value, block))`
    /// on match, `None` otherwise.
    pub(super) fn try_emit_built_in_call_expression(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> Option<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let Expression::MemberAccessExpression(access) = call.operand() else {
            return None;
        };
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AbiDecode) => Some(self.emit_abi_decode(call, arguments, block)),
            _ => None,
        }
    }

    /// Emits a member access expression as an EVM intrinsic.
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// and lowers it to the matching `sol.*` operation. Address-base intrinsics
    /// (`address.balance`, `address.codehash`, `address.code`) first evaluate
    /// the address operand and pass it as the operation's container address.
    pub(super) fn emit_built_in_member_access(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    BalanceOperation::builder(context.mlir_context, context.location())
                        .cont_addr(address_value)
                        .out(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCodehash) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    CodeHashOperation::builder(context.mlir_context, context.location())
                        .cont_addr(address_value)
                        .out(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCode) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    CodeOperation::builder(context.mlir_context, context.location())
                        .cont_addr(address_value)
                        .out(AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir())
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::Length) => self.emit_unary_member_intrinsic(access, block, |operand| {
                LengthOperation::builder(context.mlir_context, context.location())
                    .inp(operand)
                    .len(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                    .build()
                    .into()
            }),
            Some(BuiltIn::AddressSend) => {
                let arguments = arguments.expect("send is a member-access call");
                let BlockAnd { value: addr, block } =
                    access.operand().emit(self.expression_context, block);
                let (values, block) = self.emit_argument_values(arguments, block);
                let value = block
                    .append_operation(
                        SendOperation::builder(context.mlir_context, context.location())
                            .addr(addr)
                            .val(values[0])
                            .status(AstType::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("send always produces one result")
                    .into();
                (Some(value), block)
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                let BlockAnd { value: addr, block } =
                    access.operand().emit(self.expression_context, block);
                let (values, block) = self.emit_argument_values(arguments, block);
                block.append_operation(
                    TransferOperation::builder(context.mlir_context, context.location())
                        .addr(addr)
                        .val(values[0])
                        .build()
                        .into(),
                );
                (None, block)
            }
            Some(BuiltIn::AbiEncode) => {
                let arguments = arguments.expect("abi.encode is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block);
                let result = self.emit_sol_encode(&values, None, false, &block);
                (Some(result), block)
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let arguments = arguments.expect("abi.encodePacked is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block);
                let result = self.emit_sol_encode(&values, None, true, &block);
                (Some(result), block)
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let arguments = arguments.expect("abi.encodeWithSelector is a member-access call");
                let (mut values, block) = self.emit_argument_values(arguments, block);
                let selector =
                    AstValue::new(values.remove(0)).cast(AstType::new(AstType::fixed_bytes(context.mlir_context, 4).into_mlir()), context, &block).into_mlir();
                let result = self.emit_sol_encode(&values, Some(selector), false, &block);
                (Some(result), block)
            }
            Some(BuiltIn::AbiEncodeWithSignature) => {
                let arguments = arguments.expect("abi.encodeWithSignature is a member-access call");
                let mut iter = arguments.iter();
                let signature_expression =
                    iter.next().expect("slang validates non-empty arguments");
                let Expression::StringExpression(string_expression) = signature_expression else {
                    unimplemented!(
                        "abi.encodeWithSignature with a non-literal signature is not yet supported"
                    );
                };
                let signature_bytes = string_expression.value();
                let hash = solx_utils::Keccak256Hash::from_slice(&signature_bytes);
                let selector_bytes: [u8; 4] = hash.as_bytes()[..4]
                    .try_into()
                    .expect("keccak256 always yields 32 bytes");
                let selector_word = u32::from_be_bytes(selector_bytes);
                let selector_int = AstValue::constant(
                    i64::from(selector_word),
                    AstType::new(Type::from(IntegerType::unsigned(context.mlir_context, 32))),
                    context,
                    &block,
                ).into_mlir();
                let selector_value = block
                    .append_operation(
                        BytesCastOperation::builder(context.mlir_context, context.location())
                            .inp(selector_int)
                            .out(AstType::fixed_bytes(context.mlir_context, 4).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.bytes_cast always produces one result")
                    .into();
                let mut values = Vec::with_capacity(arguments.len() - 1);
                let mut current = block;
                for argument in iter {
                    let BlockAnd { value, block: next } =
                        argument.emit(self.expression_context, current);
                    values.push(value);
                    current = next;
                }
                let result = self.emit_sol_encode(&values, Some(selector_value), false, &current);
                (Some(result), current)
            }
            Some(BuiltIn::BytesConcat | BuiltIn::StringConcat) => {
                let arguments = arguments.expect("bytes.concat / string.concat is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block);
                let result = block
                    .append_operation(
                        ConcatOperation::builder(context.mlir_context, context.location())
                            .args(&values)
                            .result(AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.concat always produces one result")
                    .into();
                (Some(result), block)
            }
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, block)
            }
            resolved => {
                let operation = match resolved {
                    Some(BuiltIn::TxOrigin) => {
                        OriginOperation::builder(context.mlir_context, context.location())
                            .addr(AstType::address(context.mlir_context, false).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::TxGasPrice) => {
                        GasPriceOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgSender) => {
                        CallerOperation::builder(context.mlir_context, context.location())
                            .addr(AstType::address(context.mlir_context, false).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgValue) => {
                        CallValueOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockTimestamp) => {
                        TimestampOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockNumber) => {
                        BlockNumberOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockCoinbase) => {
                        CoinbaseOperation::builder(context.mlir_context, context.location())
                            .addr(AstType::address(context.mlir_context, false).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockChainid) => {
                        ChainIdOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockBasefee) => {
                        BaseFeeOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockGaslimit) => {
                        GasLimitOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockBlobbasefee) => {
                        BlobBaseFeeOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockDifficulty) => {
                        DifficultyOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockPrevrandao) => {
                        PrevRandaoOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgSig) => {
                        SigOperation::builder(context.mlir_context, context.location())
                            .val(AstType::fixed_bytes(context.mlir_context, 4).into_mlir())
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgData) => {
                        GetCallDataOperation::builder(context.mlir_context, context.location())
                            .addr(AstType::string(context.mlir_context, solx_utils::DataLocation::CallData).into_mlir())
                            .build()
                            .into()
                    }
                    _ => unimplemented!("unsupported member access: {}", access.member().name()),
                };
                let value = block
                    .append_operation(operation)
                    .result(0)
                    .expect("intrinsic always produces one result")
                    .into();
                (Some(value), block)
            }
        }
    }

    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let BlockAnd { value: array_value, block } =
            access.operand().emit(self.expression_context, block);
        let context = self.expression_context.state;
        AstValue::new(array_value).pop(context, &block);
        (None, block)
    }

    /// Emits `arr.push(x)` / `arr.push()` / `bytes.push()` as `sol.push`,
    /// followed by `sol.store` of the cast value when one is provided.
    /// Returns the new slot reference for the no-arg form, otherwise `None`.
    fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .expect("base of array push has a resolved type");
        let value_argument = arguments.iter().next();
        let context = self.expression_context.state;
        if let (SlangType::Bytes(_), Some(value_argument)) = (&base_slang_type, &value_argument) {
            let byte_type = AstType::fixed_bytes(context.mlir_context, 1);
            let BlockAnd { value: bytes_reference, block } =
                base.emit(self.expression_context, block);
            let BlockAnd { value, block } = value_argument.emit(self.expression_context, block);
            let byte_value = AstValue::new(value).bytes_cast(byte_type, context, &block);
            block.append_operation(
                PushStringOperation::builder(context.mlir_context, context.location())
                    .addr(bytes_reference)
                    .value(byte_value.into_mlir())
                    .build()
                    .into(),
            );
            return (None, block);
        }

        let (element_type, slang_location) = match &base_slang_type {
            SlangType::Array(array_type) => (
                TypeConversion::resolve_slang_type(&array_type.element_type(), None, context),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (AstType::fixed_bytes(context.mlir_context, 1).into_mlir(), bytes_type.location()),
            other => unreachable!(
                "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let base_location = match slang_location {
            SlangDataLocation::Inherited => {
                unreachable!("slang's binder should not surface Inherited at an array push base")
            }
            other => solx_utils::DataLocation::from_slang(other, None),
        };

        let BlockAnd { value: array_value, block } = base.emit(self.expression_context, block);
        let address_type = AstType::pointer(context.mlir_context, element_type, base_location).into_mlir();
        let new_slot = AstValue::new(array_value).push(AstType::new(address_type), context, &block).into_mlir();

        let Some(value_argument) = value_argument else {
            return (Some(new_slot), block);
        };
        let BlockAnd { value, block } = value_argument.emit(self.expression_context, block);
        let cast_value =
            TypeConversion::from_target_type(element_type, context).emit(value, context, &block);
        Pointer::new(new_slot).store(AstValue::new(cast_value), context, &block);
        (None, block)
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
        let BlockAnd { value: address_value, block } =
            access.operand().emit(self.expression_context, block);
        let value = block
            .append_operation(build_op(address_value))
            .result(0)
            .expect("unary member intrinsic always produces one result")
            .into();
        (Some(value), block)
    }

    /// Emits a `sol.encode` operation producing a `bytes memory` payload.
    ///
    /// `selector`, when present, is prepended as the first 4 bytes of the
    /// payload and must already be of `!sol.fixed_bytes<4>` type. `packed`
    /// emits the ABI-packed encoding (no per-element padding).
    ///
    /// Sets `operand_segment_sizes` manually because melior's ODS-generated
    /// builder does not synthesize the attribute for `AttrSizedOperandSegments`
    /// ops; the dialect verifier rejects the op without it.
    fn emit_sol_encode(
        &self,
        ins: &[Value<'context, 'block>],
        selector: Option<Value<'context, 'block>>,
        packed: bool,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let context = self.expression_context.state;
        let mut op_builder = EncodeOperation::builder(context.mlir_context, context.location())
            .ins(ins)
            .res(AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir());
        if let Some(selector_value) = selector {
            op_builder = op_builder.selector(selector_value);
        }
        if packed {
            op_builder = op_builder.packed(Attribute::unit(context.mlir_context));
        }
        let mut operation: Operation = op_builder.build().into();
        let ins_count = i32::try_from(ins.len()).expect("encode argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(
            context.mlir_context,
            &[ins_count, i32::from(selector.is_some())],
        );
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        block
            .append_operation(operation)
            .result(0)
            .expect("sol.encode always produces one result")
            .into()
    }

    /// Emits `abi.decode(payload, (T))` as a `sol.decode` operation.
    ///
    /// The result type comes from the call's slang type (`call.get_type()`);
    /// multi-result decode requires the multi-result-call dispatch and is
    /// not yet supported.
    fn emit_abi_decode(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("slang validates the payload argument");
        let BlockAnd { value: payload_value, block } =
            payload_expression.emit(self.expression_context, block);
        let return_slang_type = call
            .get_type()
            .expect("abi.decode call is typed by the binder");
        if matches!(return_slang_type, SlangType::Tuple(_)) {
            unimplemented!("abi.decode returning multiple values is not yet supported");
        }
        let context = self.expression_context.state;
        let result_type = TypeConversion::resolve_slang_type(&return_slang_type, None, context);
        let value = block
            .append_operation(
                DecodeOperation::builder(context.mlir_context, context.location())
                    .addr(payload_value)
                    .outs(&[result_type])
                    .build()
                    .into(),
            )
            .result(0)
            .expect("abi.decode single-result always produces one value")
            .into();
        (value, block)
    }

    /// Emits `addr.call(data)` / `addr.delegatecall(data)` / `addr.staticcall(data)` as the matching
    /// `sol.bare_*` operation, yielding the `(bool success, bytes memory returndata)` pair.
    ///
    /// The receiver evaluates to an address and the single payload argument is copied into memory,
    /// since the operation's input rejects a storage- or calldata-sourced operand. A `{gas: g}` option
    /// selects the forwarded gas, defaulting to `sol.gasleft`; a `{value: v}` option selects the wei a
    /// plain `call` sends, defaulting to zero.
    pub(super) fn emit_bare_call(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let BlockAnd { value: address, block } =
            access.operand().emit(self.expression_context, block);
        let argument = arguments.iter().next().expect("slang validates the payload argument");
        let BlockAnd { value: input, block } = argument.emit(self.expression_context, block);
        let input = AstValue::new(input)
            .data_loc_cast(AstType::string(context.mlir_context, solx_utils::DataLocation::Memory), context, &block)
            .into_mlir();
        let gas = call_gas.unwrap_or_else(|| {
            block
                .append_operation(
                    GasLeftOperation::builder(context.mlir_context, context.location())
                        .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                        .build()
                        .into(),
                )
                .result(0)
                .expect("gasleft always produces one result")
                .into()
        });
        let status_type =
            AstType::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let ret_data_type =
            AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir();
        let operation: Operation = match kind {
            BuiltIn::AddressCall => {
                let value = call_value.unwrap_or_else(|| {
                    AstValue::constant(
                        0,
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                        context,
                        &block,
                    )
                    .into_mlir()
                });
                BareCallOperation::builder(context.mlir_context, context.location())
                    .addr(address)
                    .gas(gas)
                    .val(value)
                    .inp(input)
                    .status(status_type)
                    .ret_data(ret_data_type)
                    .build()
                    .into()
            }
            BuiltIn::AddressDelegatecall => {
                BareDelegateCallOperation::builder(context.mlir_context, context.location())
                    .addr(address)
                    .gas(gas)
                    .inp(input)
                    .status(status_type)
                    .ret_data(ret_data_type)
                    .build()
                    .into()
            }
            BuiltIn::AddressStaticcall => {
                BareStaticCallOperation::builder(context.mlir_context, context.location())
                    .addr(address)
                    .gas(gas)
                    .inp(input)
                    .status(status_type)
                    .ret_data(ret_data_type)
                    .build()
                    .into()
            }
            _ => unreachable!("bare call kind is call, delegatecall, or staticcall"),
        };
        let operation = block.append_operation(operation);
        let status = operation
            .result(0)
            .expect("a bare call always produces a status")
            .into();
        let ret_data = operation
            .result(1)
            .expect("a bare call always produces return data")
            .into();
        (vec![status, ret_data], block)
    }
}
