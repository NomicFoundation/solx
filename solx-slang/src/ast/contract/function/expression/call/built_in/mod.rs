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

/// The specification of an external contract call (`sol.ext_icall`): the callee
/// (`receiver`, cast to `address`), its `selector` and ABI (`parameter_types` /
/// `return_types`), the evaluated `argument_values`, the forwarded wei
/// `call_value` (zero when `None`), and whether it lowers to `STATICCALL`.
/// Bundled so the call emitter takes one spec plus the target block rather than
/// eight parameters.
#[derive(Clone, Copy)]
struct ExternalCall<'a, 'context, 'block> {
    /// The callee, cast to an `address` before the call.
    receiver: Value<'context, 'block>,
    /// The callee's 4-byte function selector.
    selector: u32,
    /// The callee's parameter types (drive ABI encoding of the arguments).
    parameter_types: &'a [Type<'context>],
    /// The callee's return types (drive ABI decoding of the results).
    return_types: &'a [Type<'context>],
    /// The already-evaluated, coerced argument values.
    argument_values: &'a [Value<'context, 'block>],
    /// The forwarded wei value; `None` lowers to a zero constant.
    call_value: Option<Value<'context, 'block>>,
    /// `true` lowers to `STATICCALL` (a `view`/`pure` callee).
    static_call: bool,
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
    // A flat per-built-in dispatch: one `match built_in` arm per [`BuiltIn`]
    // variant, each a thin op emission of the same shape (eval args, build one
    // operation, return its result). The line count is inherent to the number
    // of built-ins, not nested logic, so it is allowed rather than split.
    #[allow(clippy::too_many_lines)]
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
        call: &ExternalCall<'_, 'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>> {
        let ExternalCall {
            receiver,
            selector,
            parameter_types,
            return_types,
            argument_values,
            call_value,
            static_call,
        } = *call;
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
    ///
    /// Tries each specialised member-access handler in turn (wrap/unwrap, enum
    /// variant, function pointers, this/local/external calls, type queries),
    /// then falls back to the per-built-in intrinsic dispatch.
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

        self.emit_built_in_member_intrinsic(access, arguments, call_value, block)
    }

    /// Dispatches a built-in member access to its intrinsic emitter once the
    /// specialised handlers in [`Self::emit_built_in_member_access_with_value`]
    /// have declined: address members (`balance`/`codehash`/`code`/`length`,
    /// `send`/`transfer`, bare calls), `abi.encode*`, array `push`/`pop`, and
    /// the environment globals.
    fn emit_built_in_member_intrinsic(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        call_value: Option<Value<'context, 'block>>,
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
                self.emit_address_send(access, arguments, block)
            }
            Some(BuiltIn::AddressTransfer) => {
                let arguments = arguments.expect("transfer is a member-access call");
                self.emit_address_transfer(access, arguments, block)
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
            resolved => self.emit_environment_global(resolved, access, block),
        }
    }

    /// Emits `address.send(amount)` (`sol.send`): forwards `amount` wei and
    /// returns a `bool` success status without reverting on failure.
    fn emit_address_send(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
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

    /// Emits `address.transfer(amount)` (`sol.transfer`): reverts on failure.
    /// The wei amount is widened to `ui256` — a narrow literal such as
    /// `transfer(1 wei)` must be widened.
    fn emit_address_transfer(
        &self,
        access: &MemberAccessExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let (addr, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (values, block) = self.emit_argument_values(arguments, block)?;
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

    /// Emits a nullary environment / global intrinsic that takes no operand —
    /// `tx.origin`, `tx.gasprice`, `msg.sender`/`msg.value`/`msg.sig`/`msg.data`,
    /// and the `block.*` globals. `resolved` is the built-in the member access
    /// resolved to; an unrecognised access is an unimplemented lowering (not a
    /// program error), so it is marked with `unimplemented!`.
    fn emit_environment_global(
        &self,
        resolved: Option<BuiltIn>,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
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

    /// Emits `abi.encodeWithSignature(sig, args...)`: the 4-byte selector is the
    /// high bytes of `keccak256(sig)` — folded at compile time for a string
    /// literal, or computed at runtime (`bytes4(keccak256(sig))`) for a dynamic
    /// signature — followed by the ABI-encoded arguments.
    fn emit_abi_encode_with_signature(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let mut iter = arguments.iter();
        let signature_expression = iter.next().expect("slang validates non-empty arguments");
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

    /// Emits `abi.encodeCall(f, (args...))`: the function's 4-byte selector
    /// followed by the ABI-encoded argument tuple. The selector is statically
    /// known for a named function / getter, or pulled at runtime from a
    /// function-pointer value (`abi.encodeCall(fPointer, ...)`) via
    /// `sol.ext_func_selector`. The second argument is the call-argument tuple
    /// (possibly empty).
    fn emit_abi_encode_call(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
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

}

mod abi;
mod array;
mod bare_call;
mod library_call;
mod member_call;
mod member_reference;
mod new;
mod require;
mod type_introspection;
