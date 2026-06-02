//!
//! Solidity built-in function and EVM intrinsic lowering.
//!

pub(crate) use melior::ir::Attribute;
pub(crate) use melior::ir::BlockLike;
pub(crate) use melior::ir::BlockRef;
pub(crate) use melior::ir::Operation;
pub(crate) use melior::ir::Type;
pub(crate) use melior::ir::Value;
pub(crate) use melior::ir::ValueLike;
pub(crate) use melior::ir::attribute::StringAttribute;
pub(crate) use melior::ir::r#type::IntegerType;
pub(crate) use slang_solidity_v2::ast::BuiltIn;
pub(crate) use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
pub(crate) use slang_solidity_v2::ast::Definition;
pub(crate) use slang_solidity_v2::ast::ElementaryType;
pub(crate) use slang_solidity_v2::ast::ArgumentsDeclaration;
pub(crate) use slang_solidity_v2::ast::Expression;
pub(crate) use slang_solidity_v2::ast::FunctionCallExpression;
pub(crate) use slang_solidity_v2::ast::MemberAccessExpression;
pub(crate) use slang_solidity_v2::ast::PositionalArguments;
pub(crate) use slang_solidity_v2::ast::Type as SlangType;
pub(crate) use slang_solidity_v2::ast::TypeName as SlangTypeName;
pub(crate) use solx_mlir::ods::sol::AddModOperation;
pub(crate) use solx_mlir::ods::sol::BalanceOperation;
pub(crate) use solx_mlir::ods::sol::BareCallOperation;
pub(crate) use solx_mlir::ods::sol::ThisOperation;
pub(crate) use solx_mlir::ods::sol::BareDelegateCallOperation;
pub(crate) use solx_mlir::ods::sol::BareStaticCallOperation;
pub(crate) use solx_mlir::ods::sol::BaseFeeOperation;
pub(crate) use solx_mlir::ods::sol::BlobBaseFeeOperation;
pub(crate) use solx_mlir::ods::sol::BlockHashOperation;
pub(crate) use solx_mlir::ods::sol::BlockNumberOperation;
pub(crate) use solx_mlir::ods::sol::CallValueOperation;
pub(crate) use solx_mlir::ods::sol::CallerOperation;
pub(crate) use solx_mlir::ods::sol::ChainIdOperation;
pub(crate) use solx_mlir::ods::sol::CodeHashOperation;
pub(crate) use solx_mlir::ods::sol::CodeOperation;
pub(crate) use solx_mlir::ods::sol::CoinbaseOperation;
pub(crate) use solx_mlir::ods::sol::ConcatOperation;
pub(crate) use solx_mlir::ods::sol::DecodeOperation;
pub(crate) use solx_mlir::ods::sol::DifficultyOperation;
pub(crate) use solx_mlir::ods::sol::EcrecoverOperation;
pub(crate) use solx_mlir::ods::sol::EncodeOperation;
pub(crate) use solx_mlir::ods::sol::ExtFuncAddrOperation;
pub(crate) use solx_mlir::ods::sol::ExtFuncSelectorOperation;
pub(crate) use solx_mlir::ods::sol::GasLeftOperation;
pub(crate) use solx_mlir::ods::sol::GasLimitOperation;
pub(crate) use solx_mlir::ods::sol::GasPriceOperation;
pub(crate) use solx_mlir::ods::sol::GetCallDataOperation;
pub(crate) use solx_mlir::ods::sol::Keccak256Operation;
pub(crate) use solx_mlir::ods::sol::LengthOperation;
pub(crate) use solx_mlir::ods::sol::MulModOperation;
pub(crate) use solx_mlir::ods::sol::NewOperation;
pub(crate) use solx_mlir::ods::sol::ObjectCodeOperation;
pub(crate) use solx_mlir::ods::sol::OriginOperation;
pub(crate) use solx_mlir::ods::sol::PrevRandaoOperation;
pub(crate) use solx_mlir::ods::sol::Ripemd160Operation;
pub(crate) use solx_mlir::ods::sol::SendOperation;
pub(crate) use solx_mlir::ods::sol::Sha256Operation;
pub(crate) use solx_mlir::ods::sol::SigOperation;
pub(crate) use solx_mlir::ods::sol::TimestampOperation;
pub(crate) use solx_mlir::ods::sol::TransferOperation;

pub(crate) use crate::ast::contract::ContractEmitter;
pub(crate) use crate::ast::contract::function::expression::call::CallEmitter;
pub(crate) use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// Resolves the definition a member-access operand refers to, handling both a
/// bare identifier (`MyEnum.VARIANT`, where the operand is `MyEnum`) and a
/// qualified path (`C.MyEnum.VARIANT` / `base.MyEnum.VARIANT`, where the operand
/// is itself a `C.MyEnum` member access). Used to find the enclosing enum of a
/// qualified enum member.
fn resolve_member_access_operand(operand: &Expression) -> Option<Definition> {
    match operand {
        Expression::Identifier(identifier) => identifier.resolve_to_definition(),
        Expression::MemberAccessExpression(member_access) => {
            member_access.member().resolve_to_definition()
        }
        _ => None,
    }
}

/// Whether an external call to `function` lowers to `STATICCALL`: a `view` or
/// `pure` callee. The static type of the callee at the call site decides this
/// (e.g. a call through a `view` interface method), so it reverts if the callee
/// attempts a state change — matching solc.
fn is_static_call_mutability(function: &slang_solidity_v2::ast::FunctionDefinition) -> bool {
    matches!(
        function.mutability(),
        slang_solidity_v2::ast::FunctionMutability::View
            | slang_solidity_v2::ast::FunctionMutability::Pure
    )
}

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
            BuiltIn::Blockhash if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block)?;
                let builder = &self.expression_emitter.state.builder;
                let block_number = TypeConversion::from_target_type(
                    builder.types.ui256,
                    builder,
                )
                .emit(values[0], builder, &block);
                let value = block
                    .append_operation(
                        BlockHashOperation::builder(
                            builder.context,
                            builder.unknown_location,
                        )
                        .block_number(block_number)
                        .val(builder.types.fixed_bytes(32))
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("blockhash always produces one result")
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
            Some(BuiltIn::AbiDecode) => {
                let (values, block) = self.emit_abi_decode(call, arguments, block)?;
                let value = values
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("abi.decode produced no value"))?;
                Ok(Some((value, block)))
            }
            // `string.concat(...)` / `bytes.concat(...)` lower to `sol.concat`,
            // which takes a variadic list of string / bytesN values and yields
            // a freshly allocated memory string. An empty argument list is
            // valid (`string.concat()` -> "").
            Some(BuiltIn::StringConcat | BuiltIn::BytesConcat) => {
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
                Ok(Some((value, block)))
            }
            // `T.wrap(x)` / `T.unwrap(x)` — a user-defined value type is
            // represented as its underlying type, so both directions are pure
            // bit-level identities. Emit the single argument coerced to the
            // call's result type (the underlying MLIR type in either case).
            Some(BuiltIn::Wrap | BuiltIn::Unwrap) if arguments.len() == 1 => {
                let argument = arguments.iter().next().expect("argument count verified");
                let (value, block) = self.expression_emitter.emit_value(&argument, block)?;
                let target_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .ok_or_else(|| anyhow::anyhow!("unresolved wrap/unwrap result type"))?;
                let builder = &self.expression_emitter.state.builder;
                let value = TypeConversion::from_target_type(target_type, builder)
                    .emit(value, builder, &block);
                Ok(Some((value, block)))
            }
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
        self.emit_built_in_member_access_with_value(access, arguments, None, block)
    }

    /// Emits an external member call `recv.f(args)` (optionally with a
    /// `{value: v}` option) as a `sol.ext_icall` with `try_call` set, for
    /// `try recv.f(args) returns (...) { ... } catch { ... }`. Returns the
    /// success status, the decoded result values, and the continuation block.
    pub fn emit_external_call_try(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(Value<'context, 'block>, Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>,
    > {
        // Unwrap an optional `{value: v}` call-options layer around the
        // member access.
        let callee = call.operand();
        let mut current_block = block;
        let mut call_value: Option<Value<'context, 'block>> = None;
        let access = match &callee {
            Expression::MemberAccessExpression(access) => access.clone(),
            Expression::CallOptionsExpression(options) => {
                (call_value, current_block) = self.capture_call_value(options, current_block)?;
                match options.operand() {
                    Expression::MemberAccessExpression(access) => access,
                    // Not a try-lowerable shape → not applicable, caller falls back.
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };

        let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
            access.member().resolve_to_definition()
        else {
            return Ok(None);
        };
        let Some(selector) = function_definition.compute_selector() else {
            return Ok(None);
        };
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            &function_definition,
            &self.expression_emitter.state.builder,
        );

        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = call.arguments()
        else {
            return Ok(None);
        };

        let (receiver_value, next) = self
            .expression_emitter
            .emit_value(&access.operand(), current_block)?;
        current_block = next;
        let mut argument_values = Vec::with_capacity(positional_arguments.len());
        for argument in positional_arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        let builder = &self.expression_emitter.state.builder;
        self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
        let address =
            builder.emit_sol_address_cast(receiver_value, builder.types.sol_address, &current_block);
        let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
        let callee_ref =
            builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &current_block);
        let value = call_value
            .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &current_block));
        let (status, results) = builder.emit_sol_ext_icall_try(
            callee_ref,
            &argument_values,
            &return_types,
            value,
            &current_block,
        )?;
        Ok(Some((status, results, current_block)))
    }

    /// Emits an external call (`sol.ext_icall`) to `receiver` (cast to an
    /// address) through a `sol.ext_func_constant` built from `selector` and the
    /// callee's parameter/return types. `call_value` is the forwarded wei value
    /// (zero when `None`); `static_call` lowers to `STATICCALL`. Returns the
    /// decoded result values.
    fn emit_external_call(
        &self,
        receiver: Value<'context, 'block>,
        selector: u32,
        parameter_types: &[Type<'context>],
        return_types: &[Type<'context>],
        argument_values: &[Value<'context, 'block>],
        call_value: Option<Value<'context, 'block>>,
        static_call: bool,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>> {
        let builder = &self.expression_emitter.state.builder;
        let address = builder.emit_sol_address_cast(receiver, builder.types.sol_address, block);
        let ext_ref_type = builder.types.ext_func_ref(parameter_types, return_types);
        let callee = builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, block);
        let value =
            call_value.unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, block));
        builder.emit_sol_ext_icall(callee, argument_values, return_types, value, static_call, block)
    }

    /// As [`Self::emit_built_in_member_access`], but with an explicit external
    /// call `value` (from `f{value: v}()` call options).
    pub fn emit_built_in_member_access_with_value(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        if let Some(result) = self.try_emit_wrap_unwrap_reference(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_address_action_reference(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_enum_variant(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_function_pointer_address(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_selector(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_external_function_pointer(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) =
            self.try_emit_this_getter_call(access, arguments, call_value, block)?
        {
            return Ok(result);
        }

        if let Some(result) =
            self.try_emit_this_external_call(access, arguments, call_value, block)?
        {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_local_call(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) =
            self.try_emit_external_instance_call(access, arguments, call_value, block)?
        {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_external_getter_call(access, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_type_enum_min_max(access, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_type_min_max(access, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_type_interface_id(access, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_type_code(access, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_type_name(access, block)? {
            return Ok(result);
        }

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
                // `sol.transfer` takes the wei amount as `ui256`; a narrow
                // literal (`transfer(1 wei)`) must be widened.
                let amount = builder.emit_sol_cast(values[0], builder.types.ui256, &block);
                block.append_operation(
                    TransferOperation::builder(builder.context, builder.unknown_location)
                        .addr(addr)
                        .val(amount)
                        .build()
                        .into(),
                );
                Ok((None, block))
            }
            Some(
                builtin @ (BuiltIn::AddressCall
                | BuiltIn::AddressDelegatecall
                | BuiltIn::AddressStaticcall),
            ) => {
                let arguments = arguments.expect("bare call is a member-access call");
                let (_status, _ret_data, block) =
                    self.emit_bare_call(access, builtin, arguments, call_value, block)?;
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
                // The 4-byte selector is the high bytes of keccak256(signature):
                // folded at compile time for a string literal, or computed at
                // runtime (`bytes4(keccak256(sig))`) for a dynamic signature.
                let (selector_value, mut current) = match &signature_expression {
                    Expression::StringExpression(string_expression) => {
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
                        let selector_value = builder.emit_sol_bytes_cast(
                            selector_int,
                            builder.types.fixed_bytes(4),
                            &block,
                        );
                        (selector_value, block)
                    }
                    _ => {
                        let (signature_value, current) = self
                            .expression_emitter
                            .emit_value(&signature_expression, block)?;
                        let hash = current
                            .append_operation(
                                Keccak256Operation::builder(
                                    builder.context,
                                    builder.unknown_location,
                                )
                                .addr(signature_value)
                                .result(builder.types.fixed_bytes(32))
                                .build()
                                .into(),
                            )
                            .result(0)
                            .expect("keccak256 always produces one result")
                            .into();
                        let selector_value =
                            TypeConversion::from_target_type(builder.types.fixed_bytes(4), builder)
                                .emit(hash, builder, &current);
                        (selector_value, current)
                    }
                };
                let mut values = Vec::with_capacity(arguments.len() - 1);
                for argument in iter {
                    let (value, next) = self.expression_emitter.emit_value(&argument, current)?;
                    values.push(value);
                    current = next;
                }
                let result = self.emit_sol_encode(&values, Some(selector_value), false, &current);
                Ok((Some(result), current))
            }
            Some(BuiltIn::AbiEncodeCall) => {
                // `abi.encodeCall(f, (args...))` == the function's 4-byte
                // selector followed by the ABI-encoded argument tuple.
                let arguments = arguments.expect("abi.encodeCall is a member-access call");
                let mut iter = arguments.iter();
                let function_reference = iter
                    .next()
                    .expect("abi.encodeCall takes a function and an argument tuple");
                let function_definition = match &function_reference {
                    Expression::MemberAccessExpression(member_access) => {
                        member_access.member().resolve_to_definition()
                    }
                    Expression::Identifier(identifier) => identifier.resolve_to_definition(),
                    _ => None,
                };
                // The selector is statically known for a named function /
                // getter, or pulled at runtime from a function-pointer value
                // (`abi.encodeCall(fPointer, ...)` / `abi.encodeCall(x[0], ...)`)
                // via `sol.ext_func_selector`.
                let (selector_value, mut current) = match function_definition {
                    Some(Definition::Function(function)) => {
                        let selector_word = function.compute_selector().ok_or_else(|| {
                            anyhow::anyhow!("abi.encodeCall function has no selector")
                        })?;
                        let selector_int = builder.emit_sol_constant(
                            i64::from(selector_word),
                            Type::from(IntegerType::unsigned(builder.context, 32)),
                            &block,
                        );
                        let selector_value = builder.emit_sol_bytes_cast(
                            selector_int,
                            builder.types.fixed_bytes(4),
                            &block,
                        );
                        (selector_value, block)
                    }
                    _ => {
                        let (function_value, current) = self
                            .expression_emitter
                            .emit_value(&function_reference, block)?;
                        assert!(
                            solx_mlir::TypeFactory::is_sol_ext_function_ref(
                                function_value.r#type()
                            ),
                            "abi.encodeCall first argument must resolve to a function"
                        );
                        let selector = current
                            .append_operation(
                                ExtFuncSelectorOperation::builder(
                                    builder.context,
                                    builder.unknown_location,
                                )
                                .func(function_value)
                                .result(builder.types.fixed_bytes(4))
                                .build()
                                .into(),
                            )
                            .result(0)
                            .expect("sol.ext_func_selector always produces one result")
                            .into();
                        (selector, current)
                    }
                };
                // The second argument is the call-argument tuple (possibly empty).
                let mut values = Vec::new();
                if let Some(argument_tuple) = iter.next() {
                    match &argument_tuple {
                        Expression::TupleExpression(tuple) => {
                            for item in tuple.items().iter() {
                                if let Some(inner) = item.expression() {
                                    let (value, next) =
                                        self.expression_emitter.emit_value(&inner, current)?;
                                    values.push(value);
                                    current = next;
                                }
                            }
                        }
                        other => {
                            let (value, next) =
                                self.expression_emitter.emit_value(other, current)?;
                            values.push(value);
                            current = next;
                        }
                    }
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
                    // An unrecognised member access is an unimplemented lowering
                    // (not a program error): mark it with `unimplemented!`.
                    // TODO: split this catch-all so non-built-in member accesses (struct fields, etc.) and unimplemented built-ins are distinct.
                    _ => unimplemented!("member access lowering: {}", access.member().name()),
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

    /// Derives the `(parameter_types, return_types)` of a public state
    /// variable's auto-generated getter for the common single-level shapes — a
    /// scalar value (`x() -> T`), a single value-keyed mapping (`m(K) -> V`), or
    /// a single value-element array (`a(uint256) -> E`). Returns `None` for
    /// nested or reference-typed shapes, which fall through to the existing
    /// dispatch rather than emitting a wrong signature.
    fn getter_signature(
        &self,
        state_variable: &slang_solidity_v2::ast::StateVariableDefinition,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)> {
        let declared_type = state_variable.get_type()?;
        let builder = &self.expression_emitter.state.builder;
        match &declared_type {
            SlangType::Mapping(mapping_type) => {
                let key = mapping_type.key_type();
                let value = mapping_type.value_type();
                if key.is_reference_type() || value.is_reference_type() {
                    return None;
                }
                Some((
                    vec![TypeConversion::resolve_slang_type(&key, None, builder)],
                    vec![TypeConversion::resolve_slang_type(&value, None, builder)],
                ))
            }
            SlangType::Array(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![builder.types.ui256],
                    vec![TypeConversion::resolve_slang_type(&element, None, builder)],
                ))
            }
            SlangType::FixedSizeArray(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![builder.types.ui256],
                    vec![TypeConversion::resolve_slang_type(&element, None, builder)],
                ))
            }
            other if !other.is_reference_type() => Some((
                Vec::new(),
                vec![TypeConversion::resolve_slang_type(other, None, builder)],
            )),
            _ => None,
        }
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
        let builder = &self.expression_emitter.state.builder;

        // `bytes.push(x)` has a dedicated lowering (`sol.push_string`) that
        // handles the in-place → out-of-place encoding transition; the generic
        // `sol.push` reference path below is only for value-typed dynamic arrays
        // and the no-argument `bytes.push()` overload.
        if matches!(&base_slang_type, SlangType::Bytes(_))
            && let Some(push_value) = &value_argument
        {
            let (array_value, block) = self.expression_emitter.emit_value(&base, block)?;
            // `emit_value_for_target` materializes a string literal (`data.push("a")`)
            // as a fixedbytes constant rather than a memory string.
            let (value, block) = self
                .expression_emitter
                .emit_value_for_target(push_value, builder.types.fixed_bytes(1), block)?;
            let byte_value = TypeConversion::from_target_type(builder.types.fixed_bytes(1), builder)
                .emit(value, builder, &block);
            builder.emit_sol_push_string(array_value, byte_value, &block);
            return Ok((None, block));
        }

        let (new_slot, element_type, block) = self.emit_push_slot(access, block)?;

        let Some(value_argument) = value_argument else {
            // `arr.push()` in value position yields the freshly-appended element.
            // A value element (`uint[].push()`) is loaded from the slot (a fresh
            // default); a reference element (`uint[][].push()`) is the slot
            // reference itself, used to initialise a storage pointer.
            let builder = &self.expression_emitter.state.builder;
            if IntegerType::try_from(element_type).is_ok() {
                let loaded = builder.emit_sol_load(new_slot, element_type, &block)?;
                return Ok((Some(loaded), block));
            }
            return Ok((Some(new_slot), block));
        };
        let (value, block) = self.expression_emitter.emit_value(&value_argument, block)?;
        let builder = &self.expression_emitter.state.builder;
        let cast_value =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        builder.emit_sol_store(cast_value, new_slot, &block);
        Ok((None, block))
    }

    /// Emits `sol.push` for `arr.push()` / `bytes.push()`, returning the new
    /// element's reference, its element type, and the continued block. Shared by
    /// the value-returning push and the push-as-lvalue (`arr.push() = v`), where
    /// the caller stores the right-hand side into the returned reference.
    pub(crate) fn emit_push_slot(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Type<'context>, BlockRef<'context, 'block>)> {
        let base = access.operand();
        let base_slang_type = base
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("base of array push has no resolved type"))?;
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
        Ok((new_slot, element_type, block))
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

    /// Emits a `new Contract(args)` expression as a `sol.new` operation.
    ///
    /// The contract type comes from the binder; payability is derived the same
    /// way it is when resolving a `SlangType::Contract` reference. Value
    /// transfer (`new C{value: x}()`) and `CREATE2` salt (`new C{salt: s}()`)
    /// are not yet handled — those go through `CallOptionsExpression`.
    pub fn emit_new(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let slang_type = call.get_type();
        // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic
        // memory aggregate of `n` elements/bytes via `sol.malloc`, passing the
        // count as the `size` operand so the length slot is initialised. slang
        // resolves the array forms' call type, but `new bytes`/`new string`
        // surface no call type, so fall back to the syntactic type name (both
        // lower to a memory string).
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(TypeConversion::resolve_slang_type(
                    inner,
                    Some(solx_utils::DataLocation::Memory),
                    &self.expression_emitter.state.builder,
                ))
            }
            None
                if matches!(
                    call.operand(),
                    Expression::NewExpression(new_expression)
                        if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
                ) =>
            {
                Some(
                    self.expression_emitter
                        .state
                        .builder
                        .types
                        .string(solx_utils::DataLocation::Memory),
                )
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let (values, block) = self.emit_argument_values(arguments, block)?;
            let builder = &self.expression_emitter.state.builder;
            let address = match values.first() {
                Some(&size_value) => {
                    let size = TypeConversion::from_target_type(builder.types.ui256, builder)
                        .emit(size_value, builder, &block);
                    builder.emit_sol_malloc_sized(result_type, size, &block)
                }
                None => builder.emit_sol_malloc(result_type, &block),
            };
            return Ok((Some(address), block));
        }
        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unimplemented!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let contract_name = contract_definition.name().name();
        let payable = ContractEmitter::is_contract_payable(&contract_definition);

        // Tell the linker that this contract embeds `contract_name`'s deploy
        // bytecode so the assembler pulls it in.
        self.expression_emitter
            .state
            .add_dependency(contract_name.clone());

        let builder = &self.expression_emitter.state.builder;
        let result_type = builder.types.contract(&contract_name, payable);

        let (ctor_args, block) = self.emit_argument_values(arguments, block)?;
        let val = builder.emit_sol_constant(0, builder.types.ui256, &block);

        // `operand_segment_sizes` (TableGen order: val=1, salt=0, ctorArgs=N) is
        // synthesized by the melior op-builder macro for this
        // `AttrSizedOperandSegments` op — `.val()` and `.ctor_args()` are set while
        // the optional `salt` is left unset, yielding [1, 0, ctor_args.len()].
        let operation: Operation =
            NewOperation::builder(builder.context, builder.unknown_location)
                .obj_name(StringAttribute::new(builder.context, &contract_name))
                .val(val)
                .ctor_args(&ctor_args)
                .out(result_type)
                .build()
                .into();

        let value = block
            .append_operation(operation)
            .result(0)
            .expect("sol.new always produces one result")
            .into();
        Ok((Some(value), block))
    }

    /// Emits one of the bare-call ops and returns both `(status, ret_data)`
    /// SSA values. Gas defaults to `gasleft()`; value defaults to zero for
    /// `addr.call`. Call options (`{gas: g, value: v}`) are not yet handled.
    fn emit_bare_call(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (addr, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (input_values, block) = self.emit_argument_values(arguments, block)?;
        let input = input_values[0];

        let builder = &self.expression_emitter.state.builder;
        let gas = block
            .append_operation(
                GasLeftOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("gasleft always produces one result")
            .into();

        let operation: Operation = match kind {
            BuiltIn::AddressCall => {
                // `addr.call{value: v}(data)` forwards `v` as the CALL value;
                // a plain `addr.call(data)` (no options) sends zero. Delegate-
                // and staticcall cannot carry value, so they ignore it.
                let val = call_value
                    .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &block));
                BareCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .val(val)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            BuiltIn::AddressDelegatecall => {
                BareDelegateCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            BuiltIn::AddressStaticcall => {
                BareStaticCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            _ => unreachable!("bare call kind must be Call, Delegatecall, or Staticcall"),
        };

        let result = block.append_operation(operation);
        let status = result
            .result(0)
            .expect("bare call always produces a status")
            .into();
        let ret_data = result
            .result(1)
            .expect("bare call always produces return data")
            .into();
        Ok((status, ret_data, block))
    }

    /// Resolves the low-level bare-call kind for a member-access callee
    /// (`recv.call` / `recv.delegatecall` / `recv.staticcall`), or `None` if
    /// the member is not a low-level call.
    ///
    /// slang resolves these to a built-in for a plain `address` receiver, but
    /// fails to type a library-as-address receiver (`address(L).delegatecall`),
    /// leaving the member unresolved. In that case fall back to the member
    /// name — it is reserved for address low-level calls, and a user method of
    /// the same name would resolve to a `Definition`, so the name fallback only
    /// fires for a member that resolves to nothing at all.
    fn resolve_bare_call_kind(access: &MemberAccessExpression) -> Option<BuiltIn> {
        match access.member().resolve_to_built_in() {
            Some(
                kind @ (BuiltIn::AddressCall
                | BuiltIn::AddressDelegatecall
                | BuiltIn::AddressStaticcall),
            ) => Some(kind),
            Some(_) => None,
            None if access.member().resolve_to_definition().is_none() => {
                match access.member().name().as_str() {
                    "call" => Some(BuiltIn::AddressCall),
                    "delegatecall" => Some(BuiltIn::AddressDelegatecall),
                    "staticcall" => Some(BuiltIn::AddressStaticcall),
                    _ => None,
                }
            }
            None => None,
        }
    }

    /// Tries to emit a multi-result bare call (`addr.call`, `addr.delegatecall`,
    /// or `addr.staticcall`) used as the right-hand side of tuple
    /// deconstruction. Returns `Ok(None)` if the callee is not a bare-call
    /// member access so the caller can fall through to the named-function
    /// dispatch path.
    pub fn try_emit_bare_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        // `(bool, bytes) = recv.call{value: v}(data)` — peel an optional
        // `{value: v}` call-options layer and forward `v` as the CALL value; a
        // plain `recv.call(data)` carries no options.
        //
        // Resolve the bare-call KIND from the inner member access *before*
        // emitting anything: a non-bare-call callee (e.g. an external
        // `ins.f{value: x, gas: g()}(a)`) must leave the block untouched for the
        // caller's fallback. Emitting the option values here would double-
        // evaluate their side effects and corrupt the observable order.
        let (access, options) = match call.operand() {
            Expression::MemberAccessExpression(access) => (access, None),
            Expression::CallOptionsExpression(call_options) => match call_options.operand() {
                Expression::MemberAccessExpression(access) => (access, Some(call_options)),
                // Not a bare-call shape → not applicable, caller falls back.
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        let Some(kind) = Self::resolve_bare_call_kind(&access) else {
            return Ok(None);
        };

        // Committed to the bare call: now evaluate the options (`{value: v}`
        // captured as the CALL value, others for side effects only).
        let mut current_block = block;
        let mut call_value = None;
        if let Some(call_options) = options {
            (call_value, current_block) = self.capture_call_value(&call_options, current_block)?;
        }
        let (status, ret_data, block) =
            self.emit_bare_call(&access, kind, arguments, call_value, current_block)?;
        Ok(Some((vec![status, ret_data], block)))
    }

    /// Tries to emit an external member call `recv.f(args)` / `this.f(args)`
    /// used as the right-hand side of tuple deconstruction, returning every
    /// decoded result value in declaration order. Returns `Ok(None)` if the
    /// callee is not a member access resolving to a function, so the caller
    /// can fall through to the named-function dispatch path.
    ///
    /// Library and bare-call (`addr.call` / `delegatecall` / `staticcall`)
    /// callees are handled by earlier branches of
    /// [`super::super::call::CallEmitter::emit_function_call_results`], so this
    /// path only ever sees genuine external contract calls. The call is lowered
    /// as a real `sol.ext_icall`, which is always correct for tuple returns
    /// even when a same-contract `this.f()` could otherwise use the local-call
    /// shortcut.
    pub fn try_emit_external_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        // Unwrap an optional `{value: v}` / `{gas: g}` call-options layer. Only
        // the value is forwarded; gas is left to the backend's default stipend.
        let mut current_block = block;
        let mut call_value: Option<Value<'context, 'block>> = None;
        let access = match call.operand() {
            Expression::MemberAccessExpression(access) => access,
            Expression::CallOptionsExpression(options) => {
                (call_value, current_block) = self.capture_call_value(&options, current_block)?;
                match options.operand() {
                    Expression::MemberAccessExpression(access) => access,
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
            access.member().resolve_to_definition()
        else {
            return Ok(None);
        };
        let Some(selector) = function_definition.compute_selector() else {
            return Ok(None);
        };
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            &function_definition,
            &self.expression_emitter.state.builder,
        );

        let (receiver_value, next) =
            self.expression_emitter.emit_value(&access.operand(), current_block)?;
        current_block = next;
        let mut argument_values = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        let builder = &self.expression_emitter.state.builder;
        self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
        let address = builder.emit_sol_address_cast(
            receiver_value,
            builder.types.sol_address,
            &current_block,
        );
        let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
        let callee_ref =
            builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &current_block);
        let value = call_value
            .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &current_block));
        // A call to a `view`/`pure` function uses `STATICCALL`: the static type
        // of the callee determines this (e.g. calling through a `view` interface
        // method), so it reverts if the callee mutates state, matching solc.
        let static_call = is_static_call_mutability(&function_definition);
        let results = builder.emit_sol_ext_icall(
            callee_ref,
            &argument_values,
            &return_types,
            value,
            static_call,
            &current_block,
        )?;
        Ok(Some((results, current_block)))
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

        // `require(cond, CustomError(args))` reverts with the custom error's
        // ABI encoding (selector + encoded args) when `cond` is false, exactly
        // like `revert CustomError(args)` but gated on the condition.
        if let Some(Expression::FunctionCallExpression(error_call)) = message
            && let Expression::Identifier(callee) = error_call.operand()
            && let Some(Definition::Error(error_definition)) = callee.resolve_to_definition()
        {
            let signature = error_definition.compute_canonical_signature().ok_or_else(|| {
                anyhow::anyhow!(
                    "cannot compute canonical signature for error `{}`",
                    error_definition.name().name()
                )
            })?;
            let ArgumentsDeclaration::PositionalArguments(error_arguments) =
                error_call.arguments()
            else {
                unimplemented!("named arguments in a `require` custom error are not supported");
            };
            let (mut argument_values, block) =
                self.emit_argument_values(&error_arguments, block)?;
            let parameters = error_definition.parameters();
            let builder = &self.expression_emitter.state.builder;
            for (value, parameter) in argument_values.iter_mut().zip(parameters.iter()) {
                let parameter_type = TypeConversion::resolve_slang_type(
                    &parameter
                        .get_type()
                        .expect("error parameter typed by the binder"),
                    None,
                    builder,
                );
                *value = TypeConversion::from_target_type(parameter_type, builder).emit(
                    *value,
                    builder,
                    &block,
                );
            }
            builder.emit_sol_require(
                condition_boolean,
                Some(&signature),
                &argument_values,
                true,
                &block,
            );
            return Ok(block);
        }

        let builder = &self.expression_emitter.state.builder;
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes)
                    .map_err(|_| anyhow::anyhow!("require message contains invalid UTF-8"))?;
                builder.emit_sol_require(condition_boolean, Some(&literal), &[], false, &block);
                Ok(block)
            }
            Some(expression) => {
                let (message_value, block) =
                    self.expression_emitter.emit_value(expression, block)?;
                let string_memory_type = builder.types.string(solx_utils::DataLocation::Memory);
                let message_value = TypeConversion::from_target_type(string_memory_type, builder)
                    .emit(message_value, builder, &block);
                builder.emit_sol_require(
                    condition_boolean,
                    Some("Error(string)"),
                    &[message_value],
                    true,
                    &block,
                );
                Ok(block)
            }
            None => {
                builder.emit_sol_require(condition_boolean, None, &[], false, &block);
                Ok(block)
            }
        }
    }

    /// Emits an external/public library call `L.f(args)` as a delegatecall to
    /// the linked library: ABI-encode `(selector, args)`, `sol.lib_addr "L"` for
    /// the address, `sol.bare_delegate_call`, re-revert on failure, then decode
    /// the return value. Returns the (single) decoded result.
    pub fn emit_library_external_call(
        &self,
        library_name: &str,
        function: &slang_solidity_v2::ast::FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function, &self.expression_emitter.state.builder);
        let selector = function
            .compute_selector()
            .ok_or_else(|| anyhow::anyhow!("library function '{library_name}' has no selector"))?;

        // Evaluate and coerce the arguments to the declared parameter types.
        let mut argument_values = Vec::with_capacity(arguments.iter().count());
        let mut current_block = block;
        for (index, argument) in arguments.iter().enumerate() {
            let (value, next) = match parameter_types.get(index) {
                Some(&parameter_type) => self.expression_emitter.emit_value_for_target(
                    &argument,
                    parameter_type,
                    current_block,
                )?,
                None => self.expression_emitter.emit_value(&argument, current_block)?,
            };
            let builder = &self.expression_emitter.state.builder;
            let value = match parameter_types.get(index) {
                Some(&parameter_type) => {
                    TypeConversion::from_target_type(parameter_type, builder).emit(value, builder, &next)
                }
                None => value,
            };
            argument_values.push(value);
            current_block = next;
        }

        let builder = &self.expression_emitter.state.builder;
        // Build the calldata: the 4-byte selector followed by the ABI-encoded
        // argument tuple.
        let selector_unsigned = builder.emit_sol_constant(
            i64::from(selector),
            Type::from(IntegerType::unsigned(builder.context, 32)),
            &current_block,
        );
        let selector_bytes =
            builder.emit_sol_cast(selector_unsigned, builder.types.fixed_bytes(4), &current_block);
        let calldata =
            self.emit_sol_encode(&argument_values, Some(selector_bytes), false, &current_block);

        let builder = &self.expression_emitter.state.builder;
        let address = builder.emit_sol_lib_addr(library_name, &current_block);
        let gas = current_block
            .append_operation(
                GasLeftOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("gasleft produces one result")
            .into();
        let call = current_block.append_operation(
            BareDelegateCallOperation::builder(builder.context, builder.unknown_location)
                .addr(address)
                .gas(gas)
                .inp(calldata)
                .status(builder.types.i1)
                .ret_data(builder.types.sol_string_memory)
                .build()
                .into(),
        );
        let status = call.result(0).expect("delegatecall status").into();
        let return_data: Value<'context, 'block> =
            call.result(1).expect("delegatecall returndata").into();

        // Revert (bubbling the callee's revert data) when the call failed.
        let (then_block, else_block) = builder.emit_sol_if(status, &current_block);
        builder.emit_sol_yield(&then_block);
        Self::emit_bubble_revert(builder, &else_block);
        builder.emit_sol_yield(&else_block);

        if return_types.is_empty() {
            return Ok((None, current_block));
        }
        let decoded = current_block
            .append_operation(
                DecodeOperation::builder(builder.context, builder.unknown_location)
                    .addr(return_data)
                    .outs(&return_types)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.decode yields one result per requested type")
            .into();
        Ok((Some(decoded), current_block))
    }

    /// Emits a raw re-revert of the entire current returndata
    /// (`returndatacopy` + `revert`). `yul.revert` is not a terminator, so the
    /// caller appends a structural terminator after it.
    fn emit_bubble_revert(
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let i256 = Type::from(IntegerType::new(builder.context, 256));
        let size = block
            .append_operation(
                solx_mlir::ods::yul::ReturnDataSizeOperation::builder(
                    builder.context,
                    builder.unknown_location,
                )
                .out(i256)
                .build()
                .into(),
            )
            .result(0)
            .expect("yul.returndatasize produces one result")
            .into();
        let zero_unsigned = builder.emit_sol_constant(0, builder.types.ui256, block);
        let zero = builder.emit_sol_cast(zero_unsigned, i256, block);
        block.append_operation(
            solx_mlir::ods::yul::ReturnDataCopyOperation::builder(
                builder.context,
                builder.unknown_location,
            )
            .dst(zero)
            .src(zero)
            .size(size)
            .build()
            .into(),
        );
        block.append_operation(
            solx_mlir::ods::yul::RevertOperation::builder(builder.context, builder.unknown_location)
                .addr(zero)
                .size(size)
                .build()
                .into(),
        );
    }
}

mod abi;
mod member_call;
mod member_reference;
mod type_introspection;
