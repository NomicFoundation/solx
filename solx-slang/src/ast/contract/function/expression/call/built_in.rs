//!
//! Solidity built-in function and EVM intrinsic lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::Effect;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit `callee(arguments)` as a Solidity built-in.
    ///
    /// Resolves the callee via slang's binder to a [`BuiltIn`] variant.
    /// On match, returns `Ok(Some((value, block)))`, where `value` is
    /// `Some(...)` for value-producing built-ins and `None` for
    /// statement-style built-ins. Returns `Ok(None)` if the callee is not a
    /// built-in and the caller should fall through to generic dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is a built-in but its arguments are
    /// malformed.
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
                let value = Value::gas_left(context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Keccak256 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value = Value::keccak256(values[0], context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Sha256 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value = Value::sha256(values[0], context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Ripemd160 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value = Value::ripemd160(values[0], context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Ecrecover if arguments.len() == 4 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value =
                    Value::ecrecover(values[0], values[1], values[2], values[3], context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Addmod if arguments.len() == 3 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value = Value::addmod(values[0], values[1], values[2], context, &block);
                Ok(Some((Some(value), block)))
            }
            BuiltIn::Mulmod if arguments.len() == 3 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let context = self.expression_emitter.state;
                let value = Value::mulmod(values[0], values[1], values[2], context, &block);
                Ok(Some((Some(value), block)))
            }
            _ => Ok(None),
        }
    }

    /// Tries to emit a built-in that needs the full [`FunctionCallExpression`]
    /// context, because the result type comes from `call.get_type()` rather
    /// than from the operands.
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
                let (address, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let value = Value::balance(address, context, &block);
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressCodehash) => {
                let (address, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let value = Value::code_hash(address, context, &block);
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressCode) => {
                let (address, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let value = Value::code(address, context, &block);
                Ok((Some(value), block))
            }
            Some(BuiltIn::Length) => {
                let (aggregate, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let value = Value::length(aggregate, context, &block);
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressSend) => {
                let arguments = arguments.expect("send is a member-access call");
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let value = Value::send(addr, values[0], context, &block);
                Ok((Some(value), block))
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                let (addr, block) = self
                    .expression_emitter
                    .emit_value(&access.operand(), block)?;
                let (values, block) = self.emit_argument_values(arguments, block)?;
                Value::transfer(addr, values[0], context, &block);
                Ok((None, block))
            }
            Some(BuiltIn::AbiEncode) => {
                let arguments = arguments.expect("abi.encode is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let result = Value::encode(&values, None, false, context, &block);
                Ok((Some(result), block))
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let arguments = arguments.expect("abi.encodePacked is a member-access call");
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let result = Value::encode(&values, None, true, context, &block);
                Ok((Some(result), block))
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let arguments = arguments.expect("abi.encodeWithSelector is a member-access call");
                let (mut values, block) = self.emit_argument_values(arguments, block)?;
                let selector =
                    values
                        .remove(0)
                        .cast(Type::fixed_bytes(context.melior, 4), context, &block);
                let result = Value::encode(&values, Some(selector), false, context, &block);
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
                let selector_int = Value::constant(
                    i64::from(selector_word),
                    Type::unsigned(context.melior, solx_utils::BIT_LENGTH_X32),
                    context,
                    &block,
                );
                let selector_value =
                    selector_int.bytes_cast(Type::fixed_bytes(context.melior, 4), context, &block);
                let mut values = Vec::with_capacity(arguments.len() - 1);
                let mut current = block;
                for argument in iter {
                    let (value, next) = self.expression_emitter.emit_value(&argument, current)?;
                    values.push(value);
                    current = next;
                }
                let result = Value::encode(&values, Some(selector_value), false, context, &current);
                Ok((Some(result), current))
            }
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, block)
            }
            resolved => {
                let value = match resolved {
                    Some(BuiltIn::TxOrigin) => Value::tx_origin(context, &block),
                    Some(BuiltIn::TxGasPrice) => Value::tx_gas_price(context, &block),
                    Some(BuiltIn::MsgSender) => Value::msg_sender(context, &block),
                    Some(BuiltIn::MsgValue) => Value::msg_value(context, &block),
                    Some(BuiltIn::BlockTimestamp) => Value::block_timestamp(context, &block),
                    Some(BuiltIn::BlockNumber) => Value::block_number(context, &block),
                    Some(BuiltIn::BlockCoinbase) => Value::block_coinbase(context, &block),
                    Some(BuiltIn::BlockChainid) => Value::block_chain_id(context, &block),
                    Some(BuiltIn::BlockBasefee) => Value::block_base_fee(context, &block),
                    Some(BuiltIn::BlockGaslimit) => Value::block_gas_limit(context, &block),
                    Some(BuiltIn::BlockBlobbasefee) => Value::block_blob_base_fee(context, &block),
                    Some(BuiltIn::BlockDifficulty) => Value::block_difficulty(context, &block),
                    Some(BuiltIn::BlockPrevrandao) => Value::block_prev_randao(context, &block),
                    Some(BuiltIn::MsgSig) => Value::msg_sig(context, &block),
                    Some(BuiltIn::MsgData) => Value::msg_data(context, &block),
                    // TODO: split this catch-all so non-built-in member accesses (struct fields, etc.) and unimplemented built-ins surface distinct errors.
                    _ => anyhow::bail!("unsupported member access: {}", access.member().name()),
                };
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
        array_value.pop(context, &block);
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
            SlangType::Bytes(bytes_type) => {
                (Type::fixed_bytes(context.melior, 1), bytes_type.location())
            }
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
        let address_type = Type::pointer(context.melior, element_type, base_location);
        let new_slot = array_value.push(address_type, context, &block);

        let Some(value_argument) = value_argument else {
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let cast_value =
            TypeConversion::from_target_type(element_type, context).emit(value, context, &block);
        Place::from(new_slot).store(cast_value, context, &block);
        Ok((None, block))
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
        let value = Value::decode(payload_value, result_type, context, &block);
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
        Effect::new(context, block).assert(condition_boolean);
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
                Effect::new(context, block).require(condition_boolean, &[], Some(&literal), false);
                Ok(block)
            }
            Some(expression) => {
                let (message_value, block) =
                    self.expression_emitter.emit_value(expression, block)?;
                let string_memory_type =
                    Type::string(context.melior, solx_utils::DataLocation::Memory);
                let message_value = TypeConversion::from_target_type(string_memory_type, context)
                    .emit(message_value, context, &block);
                Effect::new(context, block).require(
                    condition_boolean,
                    &[message_value],
                    Some("Error(string)"),
                    true,
                );
                Ok(block)
            }
            None => {
                Effect::new(context, block).require(condition_boolean, &[], None, false);
                Ok(block)
            }
        }
    }
}
