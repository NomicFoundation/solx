//!
//! Solidity built-in function and EVM intrinsic lowering.
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
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::BalanceOperation;
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
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::DifficultyOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::EncodeOperation;
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::Keccak256Operation;
use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::Sha256Operation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit `callee(arguments)` as a Solidity built-in.
    ///
    /// Resolves the callee via slang's binder to a [`BuiltIn`] variant.
    /// On match, returns `Ok(Some((value, block)))`, where `value` is
    /// `Some(...)` for value-producing built-ins (e.g. `gasleft()`) and
    /// `None` for statement-style built-ins (e.g. `assert`, `require`).
    /// Returns `Ok(None)` if the callee is not a built-in and the caller
    /// should fall through to generic dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is a built-in but its arguments are
    /// malformed (e.g. non-string `require` message).
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
            BuiltIn::Assert if arguments.len() == 1 => {
                let condition = arguments.iter().next().expect("argument count verified");
                Ok(Some((None, self.emit_assert(&condition, block)?)))
            }
            BuiltIn::Require if matches!(arguments.len(), 1 | 2) => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("argument count verified");
                let message = match iter.next() {
                    Some(Expression::StringExpression(string_expression)) => {
                        let bytes = string_expression.value();
                        Some(String::from_utf8(bytes).expect("require message is valid UTF-8"))
                    }
                    Some(_) => anyhow::bail!("require message must be a string literal"),
                    None => None,
                };
                Ok(Some((
                    None,
                    self.emit_require(&condition, message.as_deref(), block)?,
                )))
            }
            BuiltIn::Gasleft if arguments.is_empty() => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Keccak256 if arguments.len() == 1 => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Sha256 if arguments.len() == 1 => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Ripemd160 if arguments.len() == 1 => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Ecrecover if arguments.len() == 4 => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Addmod if arguments.len() == 3 => {
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
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Mulmod if arguments.len() == 3 => {
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
                Ok(Some((Some(value), block)))
            }
            _ => Ok(None),
        }
    }

    /// Tries to emit a built-in that needs the full [`FunctionCallExpression`]
    /// context — typically because the result type comes from `call.get_type()`
    /// rather than from the operands (e.g. `abi.decode(payload, (T))`).
    ///
    /// Resolves the callee's member access to a [`BuiltIn`] variant and
    /// dispatches to the matching handler. Returns `Ok(Some((value, block)))`
    /// on match, `Ok(None)` if no handler matched and the caller should
    /// fall through to other dispatch.
    pub fn try_emit_built_in_call_expression(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        let Expression::MemberAccessExpression(access) = call.operand() else {
            return Ok(None);
        };
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AbiDecode) => self.emit_abi_decode(call, arguments, block).map(Some),
            _ => Ok(None),
        }
    }

    /// Emits a member access expression as an EVM intrinsic.
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// and lowers it to the matching `sol.*` operation. Address-base intrinsics
    /// (`address.balance`, `address.codehash`, `address.code`) first evaluate
    /// the address operand and pass it as the operation's container address.
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
        let builder = &self.expression_emitter.state.builder;
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    BalanceOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.ui256)
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCodehash) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    CodeHashOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.ui256)
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::AddressCode) => {
                self.emit_unary_member_intrinsic(access, block, |address_value| {
                    CodeOperation::builder(builder.context, builder.unknown_location)
                        .cont_addr(address_value)
                        .out(builder.types.sol_string_memory)
                        .build()
                        .into()
                })
            }
            Some(BuiltIn::Length) => self.emit_unary_member_intrinsic(access, block, |operand| {
                LengthOperation::builder(builder.context, builder.unknown_location)
                    .inp(operand)
                    .len(builder.types.ui256)
                    .build()
                    .into()
            }),
            Some(BuiltIn::AddressSend) => {
                let arguments = arguments.expect("send is a member-access call");
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let value = block
                    .append_operation(
                        SendOperation::builder(builder.context, builder.unknown_location)
                            .addr(addr)
                            .val(values[0])
                            .status(builder.types.i1)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("send always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
                block.append_operation(
                    TransferOperation::builder(builder.context, builder.unknown_location)
                        .addr(addr)
                        .val(values[0])
                        .build()
                        .into(),
                );
                Ok((None, block))
            }
            Some(BuiltIn::AbiEncode) => {
                let arguments = arguments.expect("abi.encode is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let result = self.emit_sol_encode(&values, None, false, &block);
                Ok((Some(result), block))
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let arguments = arguments.expect("abi.encodePacked is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let result = self.emit_sol_encode(&values, None, true, &block);
                Ok((Some(result), block))
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let arguments = arguments.expect("abi.encodeWithSelector is a member-access call");
                let (mut values, block) = self.emit_argument_values(arguments, block)?;
                let selector =
                    builder.emit_sol_cast(values.remove(0), builder.types.fixed_bytes(4), &block);
                let result = self.emit_sol_encode(&values, Some(selector), false, &block);
                Ok((Some(result), block))
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
                let selector_int = builder.emit_sol_constant(
                    i64::from(selector_word),
                    Type::from(IntegerType::unsigned(builder.context, 32)),
                    &block,
                );
                let selector_value = block
                    .append_operation(
                        BytesCastOperation::builder(builder.context, builder.unknown_location)
                            .inp(selector_int)
                            .out(builder.types.fixed_bytes(4))
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.bytes_cast always produces one result")
                    .into();
                let mut values = Vec::with_capacity(arguments.len() - 1);
                let mut current = block;
                for argument in iter {
                    let (value, next) = self.expression_emitter.emit_value(&argument, current)?;
                    values.push(value);
                    current = next;
                }
                let result = self.emit_sol_encode(&values, Some(selector_value), false, &current);
                Ok((Some(result), current))
            }
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, block)
            }
            resolved => {
                let operation = match resolved {
                    Some(BuiltIn::TxOrigin) => {
                        OriginOperation::builder(builder.context, builder.unknown_location)
                            .addr(builder.types.sol_address)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::TxGasPrice) => {
                        GasPriceOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgSender) => {
                        CallerOperation::builder(builder.context, builder.unknown_location)
                            .addr(builder.types.sol_address)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgValue) => {
                        CallValueOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockTimestamp) => {
                        TimestampOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockNumber) => {
                        BlockNumberOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockCoinbase) => {
                        CoinbaseOperation::builder(builder.context, builder.unknown_location)
                            .addr(builder.types.sol_address)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockChainid) => {
                        ChainIdOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockBasefee) => {
                        BaseFeeOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockGaslimit) => {
                        GasLimitOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockBlobbasefee) => {
                        BlobBaseFeeOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockDifficulty) => {
                        DifficultyOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::BlockPrevrandao) => {
                        PrevRandaoOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.ui256)
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgSig) => {
                        SigOperation::builder(builder.context, builder.unknown_location)
                            .val(builder.types.fixed_bytes(4))
                            .build()
                            .into()
                    }
                    Some(BuiltIn::MsgData) => {
                        GetCallDataOperation::builder(builder.context, builder.unknown_location)
                            .addr(builder.types.string(solx_utils::DataLocation::CallData))
                            .build()
                            .into()
                    }
                    // TODO: split this catch-all so non-built-in member accesses (struct fields, etc.) and unimplemented built-ins surface distinct errors.
                    _ => anyhow::bail!("unsupported member access: {}", access.member().name()),
                };
                let value = block
                    .append_operation(operation)
                    .result(0)
                    .expect("intrinsic always produces one result")
                    .into();
                Ok((Some(value), block))
            }
        }
    }

    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (array_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        self.expression_emitter
            .state
            .builder
            .emit_sol_pop(array_value, &block);
        Ok((None, block))
    }

    /// Emits `arr.push(x)` / `arr.push()` / `bytes.push()` as `sol.push`,
    /// followed by `sol.store` of the cast value when one is provided.
    /// Returns the new slot reference for the no-arg form, otherwise `None`.
    fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of array push has no resolved type"))?;
        let value_argument = arguments.iter().next();
        if value_argument.is_some() && matches!(&base_slang_type, SlangType::Bytes(_)) {
            unimplemented!("bytes.push(x) lowers to sol.push_string, which is not yet wired");
        }
        let builder = &self.expression_emitter.state.builder;

        let (element_type, slang_location) = match &base_slang_type {
            SlangType::Array(array_type) => (
                TypeConversion::resolve_slang_type(&array_type.element_type(), None, builder),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (builder.types.fixed_bytes(1), bytes_type.location()),
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

        let (array_value, block) = self.expression_emitter.emit_value(&base, block)?;
        let address_type = builder.types.pointer(element_type, base_location);
        let new_slot = builder.emit_sol_push(array_value, address_type, &block);

        let Some(value_argument) = value_argument else {
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let cast_value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(cast_value, new_slot, &block);
        Ok((None, block))
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

    /// Emits each positional argument and returns the resulting values
    /// alongside the current block.
    fn emit_argument_values(
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
        let builder = &self.expression_emitter.state.builder;
        let mut op_builder = EncodeOperation::builder(builder.context, builder.unknown_location)
            .ins(ins)
            .res(builder.types.sol_string_memory);
        if let Some(selector_value) = selector {
            op_builder = op_builder.selector(selector_value);
        }
        if packed {
            op_builder = op_builder.packed(Attribute::unit(builder.context));
        }
        let mut operation: Operation = op_builder.build().into();
        // TODO: drop this manual segment-sizes plumbing once the melior op-builder
        // macro emits `operand_segment_sizes` automatically for ops with variadic
        // or optional operand groups.
        let ins_count = i32::try_from(ins.len()).expect("encode argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(
            builder.context,
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
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("slang validates the payload argument");
        let (payload_value, block) = self
            .expression_emitter
            .emit_value(&payload_expression, block)?;
        let return_slang_type = call
            .get_type()
            .expect("abi.decode call is typed by the binder");
        if matches!(return_slang_type, SlangType::Tuple(_)) {
            unimplemented!("abi.decode returning multiple values is not yet supported");
        }
        let builder = &self.expression_emitter.state.builder;
        let result_type = TypeConversion::resolve_slang_type(&return_slang_type, None, builder);
        let value = block
            .append_operation(
                DecodeOperation::builder(builder.context, builder.unknown_location)
                    .addr(payload_value)
                    .outs(&[result_type])
                    .build()
                    .into(),
            )
            .result(0)
            .expect("abi.decode single-result always produces one value")
            .into();
        Ok((value, block))
    }

    /// Emits an `assert(condition)` built-in via `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition_boolean, &block);
        Ok(block)
    }

    /// Emits a `require(condition)` or `require(condition, "message")` built-in
    /// via `sol.require`.
    fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&str>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_require(condition_boolean, message, &block);
        Ok(block)
    }
}
