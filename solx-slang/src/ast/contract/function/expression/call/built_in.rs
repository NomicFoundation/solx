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
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::AssertOperation;
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
use solx_mlir::ods::sol::RequireOperation;
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
                let message = iter.next();
                Ok(Some((
                    None,
                    self.emit_require(&condition, message.as_ref(), block)?,
                )))
            }
            BuiltIn::Gasleft if arguments.is_empty() => {
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        GasLeftOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        Keccak256Operation::builder(context.mlir_context, context.location())
                            .addr(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        Sha256Operation::builder(context.mlir_context, context.location())
                            .data(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        Ripemd160Operation::builder(context.mlir_context, context.location())
                            .data(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 20).into_mlir())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        EcrecoverOperation::builder(context.mlir_context, context.location())
                            .hash(values[0])
                            .v(values[1])
                            .r(values[2])
                            .s(values[3])
                            .result(AstType::address(context.mlir_context, false).into_mlir())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        AddModOperation::builder(context.mlir_context, context.location())
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
                let context = self.expression_emitter.state;
                let value = block
                    .append_operation(
                        MulModOperation::builder(context.mlir_context, context.location())
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
        let context = self.expression_emitter.state;
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
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
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
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
                block.append_operation(
                    TransferOperation::builder(context.mlir_context, context.location())
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
                    AstValue::new(values.remove(0)).cast(AstType::new(AstType::fixed_bytes(context.mlir_context, 4).into_mlir()), context, &block).into_mlir();
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
        let context = self.expression_emitter.state;
        AstValue::new(array_value).pop(context, &block);
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
        let context = self.expression_emitter.state;

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

        let (array_value, block) = self.expression_emitter.emit_value(&base, block)?;
        let address_type = AstType::pointer(context.mlir_context, element_type, base_location).into_mlir();
        let new_slot = AstValue::new(array_value).push(AstType::new(address_type), context, &block).into_mlir();

        let Some(value_argument) = value_argument else {
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let cast_value =
            TypeConversion::from_target_type(element_type, context).emit(value, context, &block);
        Pointer::new(new_slot).store(AstValue::new(cast_value), context, &block);
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
        let context = self.expression_emitter.state;
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
        // TODO: drop this manual segment-sizes plumbing once the melior op-builder
        // macro emits `operand_segment_sizes` automatically for ops with variadic
        // or optional operand groups.
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
        let context = self.expression_emitter.state;
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
        let context = self.expression_emitter.state;
        mlir_op_void!(context, &block, AssertOperation.cond(condition_boolean));
        Ok(block)
    }

    /// Emits a `require(condition)` or `require(condition, message)` built-in
    /// via `sol.require`.
    ///
    /// Literal string messages lower to `sol.require %cond, "msg" : ()`. A
    /// non-literal expression evaluates at runtime and is ABI-encoded under
    /// the `Error(string)` selector via the `call` form of `sol.require`.
    fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        let context = self.expression_emitter.state;
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes).expect("require message is valid UTF-8");
                {
                    let mut operation_builder =
                        RequireOperation::builder(context.mlir_context, context.location())
                            .cond(condition_boolean)
                            .args(&[]);
                    operation_builder = operation_builder
                        .msg(melior::ir::attribute::StringAttribute::new(context.mlir_context, &literal));
                    block.append_operation(operation_builder.build().into());
                }
                Ok(block)
            }
            Some(expression) => {
                let (message_value, block) =
                    self.expression_emitter.emit_value(expression, block)?;
                let string_memory_type = AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir();
                let message_value = TypeConversion::from_target_type(string_memory_type, context)
                    .emit(message_value, context, &block);
                {
                    let mut operation_builder =
                        RequireOperation::builder(context.mlir_context, context.location())
                            .cond(condition_boolean)
                            .args(&[message_value]);
                    operation_builder = operation_builder
                        .msg(melior::ir::attribute::StringAttribute::new(context.mlir_context, "Error(string)"));
                    operation_builder = operation_builder.call(melior::ir::Attribute::unit(context.mlir_context));
                    block.append_operation(operation_builder.build().into());
                }
                Ok(block)
            }
            None => {
                {
                    let operation_builder =
                        RequireOperation::builder(context.mlir_context, context.location())
                            .cond(condition_boolean)
                            .args(&[]);
                    block.append_operation(operation_builder.build().into());
                }
                Ok(block)
            }
        }
    }
}
