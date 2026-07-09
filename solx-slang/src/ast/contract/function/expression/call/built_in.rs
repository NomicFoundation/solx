//!
//! Solidity built-in function and EVM intrinsic lowering.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context> CallEmitter<'emitter, 'state, 'context> {
    /// Tries to emit `callee(arguments)` as a Solidity built-in.
    ///
    /// Resolves the callee via slang's binder to a [`BuiltIn`] variant.
    /// On match, returns `Ok(Some(value))`, where `value` is `Some(...)` for
    /// value-producing built-ins and `None` for statement-style built-ins.
    /// Returns `Ok(None)` if the callee is not a built-in and the caller
    /// should fall through to generic dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is a built-in but its arguments are
    /// malformed.
    pub fn try_emit_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Option<Value<'context>>>> {
        let Expression::Identifier(identifier) = callee else {
            return Ok(None);
        };
        let Some(built_in) = identifier.resolve_to_built_in() else {
            return Ok(None);
        };
        match built_in {
            BuiltIn::Assert if arguments.len() == 1 => {
                let condition = arguments.iter().next().expect("argument count verified");
                self.emit_assert(&condition, context)?;
                Ok(Some(None))
            }
            BuiltIn::Require if matches!(arguments.len(), 1 | 2) => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("argument count verified");
                let message = iter.next();
                self.emit_require(&condition, message.as_ref(), context)?;
                Ok(Some(None))
            }
            BuiltIn::Gasleft if arguments.is_empty() => {
                let value = Value::gas_left(context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Keccak256 if arguments.len() == 1 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::keccak256(values[0], context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Sha256 if arguments.len() == 1 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::sha256(values[0], context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Ripemd160 if arguments.len() == 1 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::ripemd160(values[0], context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Ecrecover if arguments.len() == 4 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::ecrecover(values[0], values[1], values[2], values[3], context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Addmod if arguments.len() == 3 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::addmod(values[0], values[1], values[2], context);
                Ok(Some(Some(value)))
            }
            BuiltIn::Mulmod if arguments.len() == 3 => {
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::mulmod(values[0], values[1], values[2], context);
                Ok(Some(Some(value)))
            }
            _ => Ok(None),
        }
    }

    /// Tries to emit a built-in that needs the full [`FunctionCallExpression`]
    /// context, because the result type comes from `call.get_type()` rather
    /// than from the operands.
    ///
    /// Resolves the callee's member access to a [`BuiltIn`] variant and
    /// dispatches to the matching handler. Returns `Ok(Some(value))` on match,
    /// `Ok(None)` if no handler matched and the caller should fall through to
    /// other dispatch.
    pub fn try_emit_built_in_call_expression(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        let Expression::MemberAccessExpression(access) = call.operand() else {
            return Ok(None);
        };
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AbiDecode) => self.emit_abi_decode(call, arguments, context).map(Some),
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressBalance) => {
                let address = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let value = Value::balance(address, context);
                Ok(Some(value))
            }
            Some(BuiltIn::AddressCodehash) => {
                let address = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let value = Value::code_hash(address, context);
                Ok(Some(value))
            }
            Some(BuiltIn::AddressCode) => {
                let address = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let value = Value::code(address, context);
                Ok(Some(value))
            }
            Some(BuiltIn::Length) => {
                let aggregate = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let value = Value::length(aggregate, context);
                Ok(Some(value))
            }
            Some(BuiltIn::AddressSend) => {
                let arguments = arguments.expect("send is a member-access call");
                let addr = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let values = self.emit_argument_values(arguments, context)?;
                let value = Value::send(addr, values[0], context);
                Ok(Some(value))
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                let addr = self
                    .expression_emitter
                    .emit_value(&access.operand(), context)?;
                let values = self.emit_argument_values(arguments, context)?;
                Value::transfer(addr, values[0], context);
                Ok(None)
            }
            Some(BuiltIn::AbiEncode) => {
                let arguments = arguments.expect("abi.encode is a member-access call");
                let values = self.emit_argument_values(arguments, context)?;
                let result = Value::encode(&values, None, false, context);
                Ok(Some(result))
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let arguments = arguments.expect("abi.encodePacked is a member-access call");
                let values = self.emit_argument_values(arguments, context)?;
                let result = Value::encode(&values, None, true, context);
                Ok(Some(result))
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let arguments = arguments.expect("abi.encodeWithSelector is a member-access call");
                let mut values = self.emit_argument_values(arguments, context)?;
                let selector = values
                    .remove(0)
                    .cast(Type::fixed_bytes(context.melior, 4), context);
                let result = Value::encode(&values, Some(selector), false, context);
                Ok(Some(result))
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
                );
                let selector_value =
                    selector_int.bytes_cast(Type::fixed_bytes(context.melior, 4), context);
                let mut values = Vec::with_capacity(arguments.len() - 1);
                for argument in iter {
                    let value = self.expression_emitter.emit_value(&argument, context)?;
                    values.push(value);
                }
                let result = Value::encode(&values, Some(selector_value), false, context);
                Ok(Some(result))
            }
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, context),
            Some(BuiltIn::ArrayPush) => {
                let arguments = arguments.expect("array push is a member-access call");
                self.emit_array_push(access, arguments, context)
            }
            resolved => {
                let value = match resolved {
                    Some(BuiltIn::TxOrigin) => Value::tx_origin(context),
                    Some(BuiltIn::TxGasPrice) => Value::tx_gas_price(context),
                    Some(BuiltIn::MsgSender) => Value::msg_sender(context),
                    Some(BuiltIn::MsgValue) => Value::msg_value(context),
                    Some(BuiltIn::BlockTimestamp) => Value::block_timestamp(context),
                    Some(BuiltIn::BlockNumber) => Value::block_number(context),
                    Some(BuiltIn::BlockCoinbase) => Value::block_coinbase(context),
                    Some(BuiltIn::BlockChainid) => Value::block_chain_id(context),
                    Some(BuiltIn::BlockBasefee) => Value::block_base_fee(context),
                    Some(BuiltIn::BlockGaslimit) => Value::block_gas_limit(context),
                    Some(BuiltIn::BlockBlobbasefee) => Value::block_blob_base_fee(context),
                    Some(BuiltIn::BlockDifficulty) => Value::block_difficulty(context),
                    Some(BuiltIn::BlockPrevrandao) => Value::block_prev_randao(context),
                    Some(BuiltIn::MsgSig) => Value::msg_sig(context),
                    Some(BuiltIn::MsgData) => Value::msg_data(context),
                    // TODO: split this catch-all so non-built-in member accesses (struct fields, etc.) and unimplemented built-ins surface distinct errors.
                    _ => anyhow::bail!("unsupported member access: {}", access.member().name()),
                };
                Ok(Some(value))
            }
        }
    }

    /// Emits `arr.pop()` / `bytes.pop()` as `sol.pop`.
    fn emit_array_pop(
        &self,
        access: &MemberAccessExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        let array_value = self
            .expression_emitter
            .emit_value(&access.operand(), context)?;
        array_value.pop(context);
        Ok(None)
    }

    /// Emits `arr.push(x)` / `arr.push()` / `bytes.push()` as `sol.push`,
    /// followed by `sol.store` of the cast value when one is provided.
    /// Returns the new slot reference for the no-arg form, otherwise `None`.
    fn emit_array_push(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of array push has no resolved type"))?;
        let value_argument = arguments.iter().next();
        if value_argument.is_some() && matches!(&base_slang_type, SlangType::Bytes(_)) {
            unimplemented!("bytes.push(x) lowers to sol.push_string, which is not yet wired");
        }

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

        let array_value = self.expression_emitter.emit_value(&base, context)?;
        let address_type = Type::pointer(context.melior, element_type, base_location);
        let new_slot = array_value.push(address_type, context);

        let Some(value_argument) = value_argument else {
            return Ok(Some(new_slot));
        };
        let value = self
            .expression_emitter
            .emit_value(&value_argument, context)?;
        let cast_value =
            TypeConversion::from_target_type(element_type, context).emit(value, context);
        Place::from(new_slot).store(cast_value, context);
        Ok(None)
    }

    /// Emits each positional argument and returns the resulting values.
    fn emit_argument_values(
        &self,
        arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Vec<Value<'context>>> {
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let value = self.expression_emitter.emit_value(&argument, context)?;
            values.push(value);
        }
        Ok(values)
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let payload_expression = arguments
            .iter()
            .next()
            .expect("slang validates the payload argument");
        let payload_value = self
            .expression_emitter
            .emit_value(&payload_expression, context)?;
        let return_slang_type = call
            .get_type()
            .expect("abi.decode call is typed by the binder");
        if matches!(return_slang_type, SlangType::Tuple(_)) {
            unimplemented!("abi.decode returning multiple values is not yet supported");
        }
        let result_type = TypeConversion::resolve_slang_type(&return_slang_type, None, context);
        let value = Value::decode(payload_value, result_type, context);
        Ok(value)
    }

    /// Emits an `assert(condition)` built-in via `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let condition_value = self.expression_emitter.emit_value(condition, context)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, context);
        let block = context.current_block();
        block.assert(condition_boolean, context);
        Ok(())
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
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let condition_value = self.expression_emitter.emit_value(condition, context)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, context);
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes).expect("require message is valid UTF-8");
                let block = context.current_block();
                block.require(condition_boolean, &[], Some(&literal), false, context);
                Ok(())
            }
            Some(expression) => {
                let message_value = self.expression_emitter.emit_value(expression, context)?;
                let string_memory_type =
                    Type::string(context.melior, solx_utils::DataLocation::Memory);
                let message_value = TypeConversion::from_target_type(string_memory_type, context)
                    .emit(message_value, context);
                let block = context.current_block();
                block.require(
                    condition_boolean,
                    &[message_value],
                    Some("Error(string)"),
                    true,
                    context,
                );
                Ok(())
            }
            None => {
                let block = context.current_block();
                block.require(condition_boolean, &[], None, false, context);
                Ok(())
            }
        }
    }
}
