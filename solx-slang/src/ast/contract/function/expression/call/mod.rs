//!
//! Function call and member access expression emission.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod positional_arguments;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use num_bigint::BigInt;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::IndexAccessKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::AssertOperation;
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::PopOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::RequireOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::Sha256Operation;
use solx_mlir::ods::sol::TransferOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for FunctionCallExpression {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Emits a function call, yielding its result values in declaration order (none for a void callee,
    /// one common, several for a tuple-returning call). The resolved callee selects the shape directly.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let callee = self.operand();
        // `{value, salt}` call options are extracted by C20b; until then no value/salt is forwarded.
        let call_value: Option<Value<'context, 'block>> = None;
        let salt: Option<Value<'context, 'block>> = None;
        let arguments = self.arguments();

        // A callee resolving to a struct definition is a struct constructor (`S(a, b)` / `Lib.S(...)`):
        // allocate the struct in memory, order field initialisers by declaration, store each coerced.
        let struct_callee = match &callee {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => None,
        };
        if let Some(Definition::Struct(struct_definition)) = struct_callee {
            let result_type = AstType::resolve_optional(self.get_type(), &context.state.builder)
                .expect("slang validated");
            let member_ids: Vec<NodeId> = struct_definition
                .members()
                .iter()
                .map(|member| member.node_id())
                .collect();
            let arguments = arguments.ordered_by(&member_ids);
            let builder = &context.state.builder;
            let struct_address =
                AstValue::malloc(result_type, None, false, builder, &block).into_mlir();
            let struct_pointer = Pointer::new(struct_address);
            let mut block = block;
            for (index, (member, argument)) in struct_definition
                .members()
                .iter()
                .zip(arguments.iter())
                .enumerate()
            {
                let field_slang_type = member.get_type().expect("slang validated");
                let field_type = AstType::resolve(
                    &field_slang_type,
                    LocationPolicy::Declared(Some(DataLocation::Memory)),
                    builder,
                );
                let index_value = AstValue::constant(
                    index as i64,
                    AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
                    builder,
                    &block,
                );
                let field_address =
                    struct_pointer.gep(index_value, AstType::new(field_type), builder, &block);
                let BlockAnd {
                    value: argument_value,
                    block: next_block,
                } = argument.emit(context, block);
                block = next_block;
                let stored = argument_value.cast(AstType::new(field_type), builder, &block);
                field_address.store(stored, builder, &block);
            }
            return BlockAnd {
                value: vec![struct_address],
                block,
            };
        }

        // `T(x)` / `bytesN("…")`: an explicit 1-argument type conversion coerces
        // the argument to the call's own type.
        if self.is_type_conversion()
            && let ArgumentsDeclaration::PositionalArguments(positional) = &arguments
            && positional.len() == 1
        {
            let first = positional.iter().next().expect("slang validated");
            let target_type = AstType::resolve_optional(self.get_type(), &context.state.builder)
                .expect("slang validated");
            let BlockAnd { value, block } = first.emit_as(target_type, context, block);
            return BlockAnd {
                value: vec![value.into_mlir()],
                block,
            };
        }

        // A call through a function-pointer VALUE — a local / parameter / contract-
        // static `fp`, a struct field `s.f`, a namespace-static `C.x`, an `arr[i]`,
        // or a `(cond ? f : g)` — dispatches through the pointer the callee yields.
        // A direct `f` / `C.f`, an external `i.f` / getter `i.x`, a library `L.f`,
        // and the built-in members all resolve to a function definition or a
        // built-in (never a pointer VALUE), so they fall through to their own
        // dispatch below.
        let function_pointer_callee = match &callee {
            Expression::Identifier(identifier) => matches!(
                identifier.resolve_to_definition(),
                Some(
                    Definition::Variable(_)
                        | Definition::Parameter(_)
                        | Definition::StateVariable(_)
                )
            ),
            Expression::MemberAccessExpression(access) => {
                match access.member().resolve_to_definition() {
                    Some(Definition::StructMember(_)) => true,
                    Some(Definition::StateVariable(_)) => matches!(&access.operand(),
                    Expression::Identifier(operand)
                        if matches!(
                            operand.resolve_to_definition(),
                            Some(Definition::Contract(_))
                        )),
                    _ => false,
                }
            }
            _ => true,
        };
        if function_pointer_callee && matches!(callee.get_type(), Some(SlangType::Function(_))) {
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("named arguments on an indirect call are not supported");
            };
            let function_slang_type = callee.get_type().expect("slang validated");
            let (parameter_types, result_types) =
                AstType::function_pointer_signature(&function_slang_type, &context.state.builder);
            let BlockAnd {
                value: callee_value,
                block,
            } = callee.emit(context, block);
            let argument_expressions: Vec<Expression> = positional.iter().collect();
            let BlockAnd {
                value: argument_values,
                block,
            } = argument_expressions.emit_as(&parameter_types, context, block);
            let results = callee_value.call_indirect(
                &argument_values,
                &result_types,
                None,
                false,
                &context.state.builder,
                &block,
            );
            return BlockAnd {
                value: results,
                block,
            };
        }

        // An identifier-callee built-in (`keccak256`, `require`, …).
        if let Expression::Identifier(identifier) = &callee
            && let Some(built_in) = identifier.resolve_to_built_in()
            && matches!(
                built_in,
                BuiltIn::Assert
                    | BuiltIn::Require
                    | BuiltIn::Gasleft
                    | BuiltIn::Keccak256
                    | BuiltIn::Sha256
                    | BuiltIn::Ripemd160
                    | BuiltIn::Ecrecover
                    | BuiltIn::Addmod
                    | BuiltIn::Mulmod
            )
        {
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("a built-in takes positional arguments only");
            };
            // Only handled built-ins with a matching argument count reach here, so
            // the per-built-in argument expectations hold; `assert` / `require` are
            // statement-style and yield no value.
            return match built_in {
                BuiltIn::Assert => {
                    let condition = positional.iter().next().expect("assert has one argument");
                    let BlockAnd {
                        value: condition_value,
                        block,
                    } = condition.emit(context, block);
                    let condition_boolean = condition_value
                        .is_nonzero(&context.state.builder, &block)
                        .into_mlir();
                    mlir_op_void!(
                        &context.state.builder,
                        &block,
                        AssertOperation.cond(condition_boolean)
                    );
                    BlockAnd {
                        value: vec![],
                        block,
                    }
                }
                BuiltIn::Require => {
                    let mut iter = positional.iter();
                    let condition = iter.next().expect("require has a condition argument");
                    let message = iter.next();
                    let BlockAnd {
                        value: condition_value,
                        block,
                    } = condition.emit(context, block);
                    let condition_boolean = condition_value
                        .is_nonzero(&context.state.builder, &block)
                        .into_mlir();
                    let builder = &context.state.builder;
                    let block = match message {
                        // A literal string message lowers to `sol.require %cond, "msg"`.
                        Some(Expression::StringExpression(string_expression)) => {
                            let bytes = string_expression.value();
                            let literal =
                                String::from_utf8(bytes).expect("require message is valid UTF-8");
                            mlir_op_void!(
                                builder,
                                &block,
                                RequireOperation
                                    .cond(condition_boolean)
                                    .args(&[])
                                    .msg(StringAttribute::new(builder.context, &literal))
                            );
                            block
                        }
                        Some(expression) => {
                            // `require(cond, CustomError(args))` (Solidity ≥ 0.8.26)
                            // lowers to the `call` form of `sol.require` carrying the
                            // error's canonical signature and its ABI-encoded
                            // arguments — the same payload `revert CustomError(args)`
                            // builds, but guarded by the condition. Any other runtime
                            // expression is ABI-encoded under the `Error(string)`
                            // selector.
                            if let Expression::FunctionCallExpression(error_call) = &expression
                                && let Some(Definition::Error(error_definition)) =
                                    (match error_call.operand() {
                                        Expression::Identifier(identifier) => {
                                            identifier.resolve_to_definition()
                                        }
                                        Expression::MemberAccessExpression(access) => {
                                            access.member().resolve_to_definition()
                                        }
                                        _ => None,
                                    })
                            {
                                let signature = error_definition
                                    .compute_canonical_signature()
                                    .expect("slang validated");
                                let parameters = error_definition.parameters();
                                let ArgumentsDeclaration::PositionalArguments(error_arguments) =
                                    error_call.arguments()
                                else {
                                    unimplemented!(
                                        "named arguments in a require custom error are not yet supported"
                                    );
                                };
                                let parameter_types: Vec<_> = parameters
                                    .iter()
                                    .map(|parameter| {
                                        AstType::resolve(
                                            &parameter.get_type().expect("slang validated"),
                                            LocationPolicy::Declared(None),
                                            &context.state.builder,
                                        )
                                    })
                                    .collect();
                                let error_argument_expressions: Vec<Expression> =
                                    error_arguments.iter().collect();
                                let BlockAnd {
                                    value: argument_values,
                                    block: current_block,
                                } = error_argument_expressions.emit_as(
                                    &parameter_types,
                                    context,
                                    block,
                                );
                                let builder = &context.state.builder;
                                mlir_op_void!(
                                    builder,
                                    &current_block,
                                    RequireOperation
                                        .cond(condition_boolean)
                                        .args(&argument_values)
                                        .msg(StringAttribute::new(builder.context, &signature))
                                        .call(Attribute::unit(builder.context))
                                );
                                current_block
                            } else {
                                let BlockAnd {
                                    value: message_value,
                                    block,
                                } = expression.emit(context, block);
                                let string_memory_type = AstType::string(
                                    builder.context,
                                    solx_utils::DataLocation::Memory,
                                )
                                .into_mlir();
                                let message_value = message_value
                                    .cast(AstType::new(string_memory_type), builder, &block)
                                    .into_mlir();
                                mlir_op_void!(
                                    builder,
                                    &block,
                                    RequireOperation
                                        .cond(condition_boolean)
                                        .args(&[message_value])
                                        .msg(StringAttribute::new(builder.context, "Error(string)"))
                                        .call(Attribute::unit(builder.context))
                                );
                                block
                            }
                        }
                        None => {
                            mlir_op_void!(
                                builder,
                                &block,
                                RequireOperation.cond(condition_boolean).args(&[])
                            );
                            block
                        }
                    };
                    BlockAnd {
                        value: vec![],
                        block,
                    }
                }
                BuiltIn::Gasleft => BlockAnd {
                    value: vec![AstValue::gas_left(&context.state.builder, &block).into_mlir()],
                    block,
                },
                BuiltIn::Keccak256 => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let value = AstValue::keccak256(
                        AstValue::from(values[0]),
                        &context.state.builder,
                        &block,
                    )
                    .into_mlir();
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Sha256 => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    let value = mlir_op!(
                        builder,
                        block,
                        Sha256Operation
                            .data(values[0])
                            .result(AstType::fixed_bytes(builder.context, 32))
                    );
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Ripemd160 => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    let value = mlir_op!(
                        builder,
                        block,
                        Ripemd160Operation
                            .data(values[0])
                            .result(AstType::fixed_bytes(builder.context, 20))
                    );
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Ecrecover => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    let value = mlir_op!(
                        builder,
                        block,
                        EcrecoverOperation
                            .hash(values[0])
                            .v(values[1])
                            .r(values[2])
                            .s(values[3])
                            .result(AstType::address(builder.context, false))
                    );
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Addmod | BuiltIn::Mulmod => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    let value = if matches!(built_in, BuiltIn::Addmod) {
                        mlir_op!(
                            builder,
                            block,
                            AddModOperation.x(values[0]).y(values[1]).r#mod(values[2])
                        )
                    } else {
                        mlir_op!(
                            builder,
                            block,
                            MulModOperation.x(values[0]).y(values[1]).r#mod(values[2])
                        )
                    };
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                _ => unreachable!("only emittable identifier built-ins are gated into this arm"),
            };
        }

        // A member-access callee: a call-position built-in, a namespace-qualified
        // struct constructor, or a member call `x.f(...)`.
        if let Expression::MemberAccessExpression(access) = &callee {
            match access.member().resolve_to_built_in() {
                // `addr.call/delegatecall/staticcall(data)` → (success, returndata).
                Some(
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall),
                ) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("a bare low-level call takes positional arguments only");
                    };
                    let BlockAnd {
                        value: address,
                        block,
                    } = access.operand().emit(context, block);
                    let argument = positional.iter().next().expect("slang validated");
                    let BlockAnd {
                        value: input,
                        block,
                    } = argument.emit(context, block);
                    let builder = &context.state.builder;
                    // `sol.bare_call`'s input rejects a non-memory operand, so an
                    // argument sourced from storage / calldata is copied into memory first.
                    let input = input
                        .cast(
                            AstType::string(builder.context, solx_utils::DataLocation::Memory),
                            builder,
                            &block,
                        )
                        .into_mlir();
                    let address = address.into_mlir();
                    let status_type =
                        AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN)
                            .into_mlir();
                    let ret_data_type =
                        AstType::string(builder.context, solx_utils::DataLocation::Memory)
                            .into_mlir();
                    let operation = match kind {
                        BuiltIn::AddressCall => {
                            let value = None.unwrap_or_else(|| {
                                AstValue::uint256(0, builder, &block).into_mlir()
                            });
                            mlir_op_build!(
                                builder,
                                BareCallOperation
                                    .addr(address)
                                    .gas(AstValue::gas_left(builder, &block))
                                    .val(value)
                                    .inp(input)
                                    .status(status_type)
                                    .ret_data(ret_data_type)
                            )
                        }
                        BuiltIn::AddressDelegatecall => mlir_op_build!(
                            builder,
                            BareDelegateCallOperation
                                .addr(address)
                                .gas(AstValue::gas_left(builder, &block))
                                .inp(input)
                                .status(status_type)
                                .ret_data(ret_data_type)
                        ),
                        BuiltIn::AddressStaticcall => mlir_op_build!(
                            builder,
                            BareStaticCallOperation
                                .addr(address)
                                .gas(AstValue::gas_left(builder, &block))
                                .inp(input)
                                .status(status_type)
                                .ret_data(ret_data_type)
                        ),
                        _ => {
                            unreachable!("bare call kind must be Call, Delegatecall, or Staticcall")
                        }
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
                    return BlockAnd {
                        value: vec![status, ret_data],
                        block,
                    };
                }
                // `abi.decode(payload, (T))` — `sol.decode` to the result types the
                // call's slang type resolves to (one per requested type).
                Some(BuiltIn::AbiDecode) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("abi.decode takes positional arguments only");
                    };
                    let payload_expression = positional.iter().next().expect("slang validated");
                    let BlockAnd {
                        value: payload_value,
                        block,
                    } = payload_expression.emit(context, block);
                    // A single MLIR result type resolved from the call's binder-assigned type.
                    let result_types: Vec<Type> = AstType::resolve_result_types(
                        &self.get_type().expect("slang validated"),
                        &context.state.builder,
                    );
                    if result_types.len() > 1 {
                        unimplemented!("abi.decode returning multiple values is not yet supported");
                    }
                    let result_type = result_types
                        .into_iter()
                        .next()
                        .expect("abi.decode yields at least one result type");
                    let builder = &context.state.builder;
                    let operation = block.append_operation(
                        DecodeOperation::builder(builder.context, builder.unknown_location)
                            .addr(payload_value.into_mlir())
                            .outs(&[result_type])
                            .build()
                            .into(),
                    );
                    let value = operation
                        .result(0)
                        .expect("sol.decode yields one result")
                        .into();
                    return BlockAnd {
                        value: vec![value],
                        block,
                    };
                }
                // Any other member built-in in call position: an ABI encode, a
                // dynamic-array `push`/`pop`, an address value transfer, or a
                // `string`/`bytes` concat — dispatched on slang's typed
                // classification of the member.
                Some(member_built_in) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("a built-in member takes positional arguments only");
                    };
                    let (value, block) = match member_built_in {
                        BuiltIn::AddressSend => {
                            // `address.send(value)` → `sol.send`, yielding the success flag.
                            let builder = &context.state.builder;
                            let BlockAnd { value: addr, block } =
                                access.operand().emit(context, block);
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            let value = mlir_op!(
                                builder,
                                block,
                                SendOperation
                                    .addr(addr)
                                    .val(values[0])
                                    .status(AstType::signless(
                                        builder.context,
                                        solx_utils::BIT_LENGTH_BOOLEAN
                                    ))
                            );
                            (Some(value), block)
                        }
                        BuiltIn::AddressTransfer => {
                            // `address.transfer(value)` → `sol.transfer` (no result).
                            let builder = &context.state.builder;
                            let BlockAnd { value: addr, block } =
                                access.operand().emit(context, block);
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            mlir_op_void!(
                                builder,
                                block,
                                TransferOperation.addr(addr).val(values[0])
                            );
                            (None, block)
                        }
                        BuiltIn::AbiEncode => {
                            // `abi.encode(args)` → a standard `sol.encode`.
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            let builder = &context.state.builder;
                            let result =
                                AstValue::abi_encode(&values, None, false, builder, &block)
                                    .into_mlir();
                            (Some(result), block)
                        }
                        BuiltIn::AbiEncodePacked => {
                            // `abi.encodePacked(args)` → a packed `sol.encode`.
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            let builder = &context.state.builder;
                            let result = AstValue::abi_encode(&values, None, true, builder, &block)
                                .into_mlir();
                            (Some(result), block)
                        }
                        BuiltIn::AbiEncodeWithSelector => {
                            // `abi.encodeWithSelector(selector, args)`: cast the first
                            // argument to `bytes4` and prepend it to the payload.
                            let BlockAnd {
                                value: mut values,
                                block,
                            } = positional.emit(context, block);
                            let builder = &context.state.builder;
                            let selector = AstValue::from(values.remove(0))
                                .cast(AstType::fixed_bytes(builder.context, 4), builder, &block)
                                .into_mlir();
                            let result = AstValue::abi_encode(
                                &values,
                                Some(selector),
                                false,
                                builder,
                                &block,
                            )
                            .into_mlir();
                            (Some(result), block)
                        }
                        BuiltIn::AbiEncodeWithSignature => {
                            // `abi.encodeWithSignature(sig, args)`: hash the signature to a 4-byte
                            // selector and prepend it (a literal hashes at compile time, a runtime one via `keccak256`).
                            let mut iter = positional.iter();
                            let signature_expression = iter.next().expect("slang validated");
                            let (selector_value, mut current) = match &signature_expression {
                                Expression::StringExpression(string_expression) => {
                                    let signature_bytes = string_expression.value();
                                    let hash =
                                        solx_utils::Keccak256Hash::from_slice(&signature_bytes);
                                    let selector_bytes: [u8; 4] = hash.as_bytes()[..4]
                                        .try_into()
                                        .expect("keccak256 always yields 32 bytes");
                                    let selector_word = u32::from_be_bytes(selector_bytes);
                                    let selector_value = AstValue::selector_constant(
                                        &BigInt::from(selector_word),
                                        4,
                                        &context.state.builder,
                                        &block,
                                    )
                                    .into_mlir();
                                    (selector_value, block)
                                }
                                _ => {
                                    unimplemented!(
                                        "abi.encodeWithSignature with a non-literal signature is not yet supported"
                                    )
                                }
                            };
                            let mut values = Vec::with_capacity(positional.len() - 1);
                            for argument in iter {
                                let BlockAnd { value, block: next } =
                                    argument.emit(context, current);
                                values.push(value.into_mlir());
                                current = next;
                            }
                            let builder = &context.state.builder;
                            let result = AstValue::abi_encode(
                                &values,
                                Some(selector_value),
                                false,
                                builder,
                                &current,
                            )
                            .into_mlir();
                            (Some(result), current)
                        }
                        BuiltIn::ArrayPop => {
                            // `arr.pop()` / `bytes.pop()` → `sol.pop`.
                            let BlockAnd {
                                value: array_value,
                                block,
                            } = access.operand().emit(context, block);
                            mlir_op_void!(
                                &context.state.builder,
                                &block,
                                PopOperation.inp(array_value)
                            );
                            (None, block)
                        }
                        BuiltIn::ArrayPush => {
                            let base = access.operand();
                            let base_slang_type = base.get_type().expect("slang validated");
                            let value_argument = positional.iter().next();
                            if let (SlangType::Bytes(_), Some(value_argument)) =
                                (&base_slang_type, &value_argument)
                            {
                                // `bytes.push(x)` appends a single byte in place via `sol.push_string`;
                                // the packed element is not separately addressable, so there is no returned slot.
                                let BlockAnd {
                                    value: bytes_reference,
                                    block,
                                } = base.emit(context, block);
                                let byte_target =
                                    AstType::fixed_bytes(context.state.builder.context, 1)
                                        .into_mlir();
                                let BlockAnd { value, block } =
                                    value_argument.emit_as(byte_target, context, block);
                                let builder = &context.state.builder;
                                let byte_value = value.into_mlir();
                                mlir_op_void!(
                                    builder,
                                    &block,
                                    PushStringOperation.addr(bytes_reference).value(byte_value)
                                );
                                (None, block)
                            } else {
                                let base_slang_type =
                                    access.operand().get_type().expect("slang validated");
                                let BlockAnd {
                                    value: array_value,
                                    block,
                                } = access.operand().emit(context, block);
                                let (new_slot, element_type) = array_value.push_slot(
                                    &base_slang_type,
                                    &context.state.builder,
                                    &block,
                                );
                                let new_slot = new_slot.into_mlir();
                                let Some(value_argument) = value_argument else {
                                    // `arr.push()` in value position yields the freshly-appended element via
                                    // `sol.load` (a value element as a fresh default, a reference as its storage reference).
                                    let builder = &context.state.builder;
                                    let loaded = Pointer::new(new_slot)
                                        .load(AstType::new(element_type), builder, &block)
                                        .into_mlir();
                                    return BlockAnd {
                                        value: vec![loaded],
                                        block,
                                    };
                                };
                                if AstType::new(element_type).is_reference() {
                                    // A reference-typed element is appended by copying the source memory
                                    // aggregate into the storage slot `push` returns (a memory→storage `sol.copy`).
                                    let BlockAnd { value, block } =
                                        value_argument.emit(context, block);
                                    Pointer::new(new_slot).copy_from(
                                        value,
                                        &context.state.builder,
                                        &block,
                                    );
                                    (None, block)
                                } else {
                                    let BlockAnd { value, block } =
                                        value_argument.emit_as(element_type, context, block);
                                    Pointer::new(new_slot).store(
                                        value,
                                        &context.state.builder,
                                        &block,
                                    );
                                    (None, block)
                                }
                            }
                        }
                        BuiltIn::StringConcat | BuiltIn::BytesConcat => {
                            // `string.concat(...)` / `bytes.concat(...)` → `sol.concat` over the variadic
                            // values, yielding a fresh memory string.
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            let builder = &context.state.builder;
                            let result_type =
                                AstType::string(builder.context, solx_utils::DataLocation::Memory)
                                    .into_mlir();
                            let value = mlir_op!(
                                builder,
                                block,
                                ConcatOperation.args(&values).result(result_type)
                            );
                            (Some(value), block)
                        }
                        _ => unimplemented!(
                            "unsupported call-position member built-in: {}",
                            access.member().name()
                        ),
                    };
                    return BlockAnd {
                        value: value.into_iter().collect(),
                        block,
                    };
                }
                None => {}
            }

            unimplemented!("unsupported member call");
        }

        // `new T[](n)` / `new bytes(n)` / `new C(args)`.
        if let Expression::NewExpression(_) = &callee {
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("named arguments on a new expression are not supported");
            };
            let slang_type = self.get_type();
            // `new T[](n)` / `new bytes(n)` / `new string(n)` allocate a dynamic memory aggregate of
            // `n` via a zeroed `sol.malloc`. The array forms resolve a call type; `new bytes` / `new
            // string` surface none, so fall back to the syntactic elementary type name.
            let dynamic_result_type = match &slang_type {
                Some(
                    inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_)),
                ) => Some(AstType::resolve(
                    inner,
                    LocationPolicy::Declared(Some(DataLocation::Memory)),
                    &context.state.builder,
                )),
                None if matches!(
                    self.operand(),
                    Expression::NewExpression(new_expression)
                        if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
                ) =>
                {
                    Some(
                        AstType::string(context.state.builder.context, DataLocation::Memory)
                            .into_mlir(),
                    )
                }
                _ => None,
            };
            if let Some(result_type) = dynamic_result_type {
                let BlockAnd {
                    value: values,
                    block: current_block,
                } = positional.emit(context, block);
                let builder = &context.state.builder;
                let address = match values.first() {
                    Some(&size_value) => {
                        let size = AstValue::from(size_value)
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &current_block,
                            )
                            .into_mlir();
                        AstValue::malloc(result_type, Some(size), true, builder, &current_block)
                            .into_mlir()
                    }
                    None => AstValue::malloc(result_type, None, true, builder, &current_block)
                        .into_mlir(),
                };
                return BlockAnd {
                    value: vec![address],
                    block: current_block,
                };
            }

            // Contract creation: `new C(args)` lowers to `sol.new` (embedding `C`'s deploy bytecode);
            // record the dependency so the linker pulls the object in. `{salt: s}` selects CREATE2.
            let Some(SlangType::Contract(contract_type)) = slang_type else {
                unimplemented!("new expression has no resolved type or unsupported new target");
            };
            let Definition::Contract(contract_definition) = contract_type.definition() else {
                unreachable!("Slang ContractType always references a Contract definition");
            };
            let contract_name = contract_definition.name().name();
            let payable = contract_definition.is_payable();
            context.state.add_dependency(contract_name.clone());

            // Coerce each constructor argument to its declared parameter type so a literal materialises
            // in the parameter's representation (the deployed constructor ABI-decodes by parameter type).
            let parameter_types = contract_definition
                .constructor()
                .map(|constructor| {
                    AstType::resolve_signature(
                        &constructor,
                        LocationPolicy::Declared(None),
                        &context.state.builder,
                    )
                    .0
                })
                .unwrap_or_default();
            let ordered: Vec<Expression> = positional.iter().collect();
            let BlockAnd {
                value: ctor_args,
                block,
            } = ordered.emit_as(&parameter_types, context, block);
            let builder = &context.state.builder;
            let result_type = AstType::contract(builder.context, &contract_name, payable);
            // `new C{value: v}()` forwards `v` wei; a plain `new C()` sends zero.
            let val = match call_value {
                Some(value) => AstValue::from(value),
                None => AstValue::uint256(0, builder, &block),
            };
            let value = AstValue::create_contract(
                &contract_name,
                val,
                salt.map(AstValue::from),
                &ctor_args,
                result_type,
                builder,
                &block,
            )
            .into_mlir();
            return BlockAnd {
                value: vec![value],
                block,
            };
        }

        let Expression::Identifier(identifier) = &callee else {
            // `T[](x)`: an empty-bracket array type used as a data-location cast.
            if let Expression::IndexAccessExpression(array_type) = &callee
                && array_type.start().is_none()
                && array_type.end().is_none()
                && !matches!(array_type.kind(), IndexAccessKind::Slice)
            {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unimplemented!("named arguments on an array-type cast are not supported");
                };
                let first = positional.iter().next().expect("slang validated");
                let target_type =
                    AstType::resolve_optional(self.get_type(), &context.state.builder)
                        .expect("slang validated");
                let BlockAnd { value, block } = first.emit_as(target_type, context, block);
                return BlockAnd {
                    value: vec![value.into_mlir()],
                    block,
                };
            }
            // A function-pointer value callee (`arr[i]`, `(cond ? f : g)`) was
            // dispatched above; any other non-identifier callee is unsupported.
            unimplemented!("unsupported callee expression");
        };
        match identifier.resolve_to_definition() {
            // A direct call passes its arguments by position; ordering them against
            // the parameter ids drives the call.
            Some(Definition::Function(function_definition)) => {
                let parameter_ids: Vec<NodeId> = function_definition
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let ordered = arguments.ordered_by(&parameter_ids);
                // Virtual dispatch: a bare internal call resolving to an overridden base function is
                // routed to the most-derived override (a non-virtual callee passes through unchanged).
                let call_id = context.state.resolve_virtual(function_definition.node_id());
                let function = context.state.resolve_function(call_id);
                let BlockAnd {
                    value: argument_values,
                    block,
                } = ordered.emit_as(&function.parameter_types, context, block);
                let results = function.call(&argument_values, &context.state.builder, &block);
                BlockAnd {
                    value: results,
                    block,
                }
            }
            _ => unimplemented!(
                "callee '{}' does not resolve to a function",
                identifier.name()
            ),
        }
    }
}
