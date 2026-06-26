//!
//! Function call and member access expression emission.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod positional_arguments;
pub mod try_external_call;
pub mod try_new_expression;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::AssertOperation;
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_mlir::ods::sol::BlobHashOperation;
use solx_mlir::ods::sol::BlockHashOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::ExtCallOperation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::PopOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::RequireOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::SelfdestructOperation;
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
use crate::ast::contract::function::expression::call_options::CallOptions;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;
use crate::ast::contract::getter::GetterSignature;
use crate::ast::analysis::query::MemberAccessOperand;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for FunctionCallExpression {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Emits a function call, yielding its result values in declaration order (none for a void callee,
    /// one common, several for a tuple-returning call). The resolved callee selects the shape directly.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        // `recv.f{value: v}(args)` / `new C{value, salt}(args)`: evaluate the option list (in source
        // order) before the arguments, forwarding `value` as msg.value and `salt` as the CREATE2 salt.
        let (call_value, salt, call_gas, block, callee) = match self.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => {
                let (value, salt, gas, block) = CallOptions(&options).capture(context, block);
                (value, salt, gas, block, options.operand().unwrap_parentheses())
            }
            other => (None, None, None, block, other),
        };
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
            let arguments = arguments.ordered_by(&member_ids).expect("slang validated");
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
                unreachable!("named arguments on an indirect call are not supported");
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
                call_value,
                call_gas,
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
                    | BuiltIn::Blockhash
                    | BuiltIn::Keccak256
                    | BuiltIn::Sha256
                    | BuiltIn::Ripemd160
                    | BuiltIn::Ecrecover
                    | BuiltIn::Addmod
                    | BuiltIn::Mulmod
                    | BuiltIn::Selfdestruct
                    | BuiltIn::Blobhash
            )
        {
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unreachable!("a built-in takes positional arguments only");
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
                                let parameter_ids: Vec<NodeId> = parameters
                                    .iter()
                                    .map(|parameter| parameter.node_id())
                                    .collect();
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
                                let error_argument_expressions = error_call
                                    .arguments()
                                    .ordered_by(&parameter_ids)
                                    .expect("slang validated");
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
                BuiltIn::Blockhash => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    // `sol.blockhash` takes a `ui256` block number; coerce a narrower
                    // argument type up first.
                    let block_number = AstValue::from(values[0])
                        .cast(
                            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                            builder,
                            &block,
                        )
                        .into_mlir();
                    let value = mlir_op!(
                        builder,
                        block,
                        BlockHashOperation
                            .block_number(block_number)
                            .val(AstType::fixed_bytes(builder.context, 32))
                    );
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Blobhash => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    // `sol.blobhash` takes a `ui256` index; coerce a narrower
                    // argument type up first (mirroring `blockhash`).
                    let index = AstValue::from(values[0])
                        .cast(
                            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                            builder,
                            &block,
                        )
                        .into_mlir();
                    let value = mlir_op!(
                        builder,
                        block,
                        BlobHashOperation
                            .idx(index)
                            .val(AstType::fixed_bytes(builder.context, 32))
                    );
                    BlockAnd {
                        value: vec![value],
                        block,
                    }
                }
                BuiltIn::Selfdestruct => {
                    let BlockAnd {
                        value: values,
                        block,
                    } = positional.emit(context, block);
                    let builder = &context.state.builder;
                    // `sol.selfdestruct` takes a payable address recipient; the
                    // argument is already `address payable` per the signature.
                    let recipient = AstValue::from(values[0])
                        .cast(AstType::address(builder.context, true), builder, &block)
                        .into_mlir();
                    mlir_op_void!(builder, &block, SelfdestructOperation.recipient(recipient));
                    BlockAnd {
                        value: vec![],
                        block,
                    }
                }
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
                    // `ecrecover(bytes32 hash, uint8 v, bytes32 r, bytes32 s)`: the
                    // hash / r / s arguments keep their literal `uint256` type, but
                    // `sol.ecrecover` takes `fixedbytes<32>` for them and `ui8` for
                    // `v`. Coerce each to its signature type (matching solc).
                    let bytes32 = AstType::fixed_bytes(builder.context, 32).into_mlir();
                    let ui8 = Type::from(IntegerType::unsigned(builder.context, 8));
                    let hash = AstValue::from(values[0])
                        .cast(AstType::new(bytes32), builder, &block)
                        .into_mlir();
                    let v = AstValue::from(values[1])
                        .cast(AstType::new(ui8), builder, &block)
                        .into_mlir();
                    let r = AstValue::from(values[2])
                        .cast(AstType::new(bytes32), builder, &block)
                        .into_mlir();
                    let s = AstValue::from(values[3])
                        .cast(AstType::new(bytes32), builder, &block)
                        .into_mlir();
                    let value = mlir_op!(
                        builder,
                        block,
                        EcrecoverOperation
                            .hash(hash)
                            .v(v)
                            .r(r)
                            .s(s)
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
                    // `addmod`/`mulmod` require identical `ui256` operand/result types, but a literal
                    // operand keeps its narrow type, so widen all three to ui256.
                    let ui256 = AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir();
                    let x = AstValue::from(values[0])
                        .cast(AstType::new(ui256), builder, &block)
                        .into_mlir();
                    let y = AstValue::from(values[1])
                        .cast(AstType::new(ui256), builder, &block)
                        .into_mlir();
                    let modulus = AstValue::from(values[2])
                        .cast(AstType::new(ui256), builder, &block)
                        .into_mlir();
                    let value = if matches!(built_in, BuiltIn::Addmod) {
                        mlir_op!(builder, block, AddModOperation.x(x).y(y).r#mod(modulus))
                    } else {
                        mlir_op!(builder, block, MulModOperation.x(x).y(y).r#mod(modulus))
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
                        unreachable!("a bare low-level call takes positional arguments only");
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
                            let value = call_value.unwrap_or_else(|| {
                                AstValue::uint256(0, builder, &block).into_mlir()
                            });
                            mlir_op_build!(
                                builder,
                                BareCallOperation
                                    .addr(address)
                                    .gas(call_gas.map(AstValue::from).unwrap_or_else(|| AstValue::gas_left(builder, &block)))
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
                                .gas(call_gas.map(AstValue::from).unwrap_or_else(|| AstValue::gas_left(builder, &block)))
                                .inp(input)
                                .status(status_type)
                                .ret_data(ret_data_type)
                        ),
                        BuiltIn::AddressStaticcall => mlir_op_build!(
                            builder,
                            BareStaticCallOperation
                                .addr(address)
                                .gas(call_gas.map(AstValue::from).unwrap_or_else(|| AstValue::gas_left(builder, &block)))
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
                    // One MLIR result type per requested type, resolved from the call's binder-assigned type.
                    let result_types: Vec<Type> = AstType::resolve_result_types(
                        &self.get_type().expect("slang validated"),
                        &context.state.builder,
                    );
                    let builder = &context.state.builder;
                    // `sol.decode` requires a memory / calldata buffer; a storage `bytes` / `string`
                    // is a reference, so copy it to memory first (memory / calldata pass through).
                    let payload_value = if matches!(
                        payload_expression
                            .get_type()
                            .and_then(|payload_type| payload_type.data_location()),
                        Some(SlangDataLocation::Storage)
                    ) {
                        payload_value.cast(
                            AstType::string(builder.context, solx_utils::DataLocation::Memory),
                            builder,
                            &block,
                        )
                    } else {
                        payload_value
                    };
                    let operation = block.append_operation(
                        DecodeOperation::builder(builder.context, builder.unknown_location)
                            .addr(payload_value.into_mlir())
                            .outs(&result_types)
                            .build()
                            .into(),
                    );
                    let values = (0..result_types.len())
                        .map(|index| {
                            operation
                                .result(index)
                                .expect("sol.decode yields one result per requested type")
                                .into()
                        })
                        .collect();
                    return BlockAnd {
                        value: values,
                        block,
                    };
                }
                // `T.wrap(x)` / `T.unwrap(x)`: a single conversion to the result type.
                Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unreachable!("a UDVT wrap/unwrap takes one positional argument");
                    };
                    let argument = positional.iter().next().expect("slang validated");
                    let BlockAnd { value, block } = argument.emit(context, block);
                    let result =
                        match AstType::resolve_optional(self.get_type(), &context.state.builder) {
                            Some(result_type) => value
                                .cast(AstType::new(result_type), &context.state.builder, &block)
                                .into_mlir(),
                            None => value.into_mlir(),
                        };
                    return BlockAnd {
                        value: vec![result],
                        block,
                    };
                }
                // Any other member built-in in call position: an ABI encode, a
                // dynamic-array `push`/`pop`, an address value transfer, or a
                // `string`/`bytes` concat — dispatched on slang's typed
                // classification of the member.
                Some(member_built_in) => {
                    // `arr.pop({})` / `arr.push({})`: empty named braces on a zero-parameter
                    // array built-in are the no-argument form (`pop` discards; `push` appends
                    // and yields the new element). Handle them directly; every other member
                    // built-in takes positional arguments only (slang rejects named args there).
                    if matches!(member_built_in, BuiltIn::ArrayPop | BuiltIn::ArrayPush)
                        && matches!(&arguments,
                            ArgumentsDeclaration::NamedArguments(named) if named.iter().next().is_none())
                    {
                        let base_slang_type =
                            access.operand().get_type().expect("slang validated");
                        let BlockAnd {
                            value: array_value,
                            block,
                        } = access.operand().emit(context, block);
                        let builder = &context.state.builder;
                        let result = match member_built_in {
                            BuiltIn::ArrayPop => {
                                mlir_op_void!(builder, &block, PopOperation.inp(array_value));
                                vec![]
                            }
                            _ => {
                                let (new_slot, element_type) =
                                    array_value.push_slot(&base_slang_type, builder, &block);
                                vec![
                                    Pointer::new(new_slot.into_mlir())
                                        .load(AstType::new(element_type), builder, &block)
                                        .into_mlir(),
                                ]
                            }
                        };
                        return BlockAnd {
                            value: result,
                            block,
                        };
                    }
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unreachable!(
                            "slang rejects named arguments on a member built-in (other than empty `pop`/`push` braces)"
                        );
                    };
                    let (value, block) = match member_built_in {
                        BuiltIn::AddressSend => {
                            // `address.send(value)` → `sol.send`, yielding the
                            // success flag. `sol.send` takes a `ui256` amount, so a
                            // narrow literal (`r.send(0)` → ui8) is widened first.
                            let builder = &context.state.builder;
                            let BlockAnd { value: addr, block } =
                                access.operand().emit(context, block);
                            let BlockAnd {
                                value: values,
                                block,
                            } = positional.emit(context, block);
                            let amount = AstValue::from(values[0])
                                .cast(
                                    AstType::unsigned(
                                        builder.context,
                                        solx_utils::BIT_LENGTH_FIELD,
                                    ),
                                    builder,
                                    &block,
                                )
                                .into_mlir();
                            let value = mlir_op!(
                                builder,
                                block,
                                SendOperation
                                    .addr(addr)
                                    .val(amount)
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
                            let amount = AstValue::from(values[0])
                                .cast(
                                    AstType::unsigned(
                                        builder.context,
                                        solx_utils::BIT_LENGTH_FIELD,
                                    ),
                                    builder,
                                    &block,
                                )
                                .into_mlir();
                            mlir_op_void!(builder, block, TransferOperation.addr(addr).val(amount));
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
                                    let BlockAnd {
                                        value: signature_value,
                                        block: current,
                                    } = signature_expression.emit(context, block);
                                    let hash = AstValue::keccak256(
                                        signature_value,
                                        &context.state.builder,
                                        &current,
                                    );
                                    let builder = &context.state.builder;
                                    let selector_value = hash
                                        .cast(
                                            AstType::fixed_bytes(builder.context, 4),
                                            builder,
                                            &current,
                                        )
                                        .into_mlir();
                                    (selector_value, current)
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
                        BuiltIn::AbiEncodeCall => {
                            // `abi.encodeCall(callee, args)`: the callee's 4-byte selector prepended to its
                            // ABI-encoded arguments. A static reference folds the selector to a constant; a
                            // runtime pointer reads it via `sol.ext_func_selector`. Arguments are coerced to
                            // the callee's parameter types, encoded from `Memory` (the external-call ABI).
                            let mut iter = positional.iter();
                            let function_expression = iter.next().expect("slang validated");
                            let call_arguments = iter.next().expect("slang validated");
                            let definition = match &function_expression {
                                Expression::MemberAccessExpression(access) => {
                                    access.member().resolve_to_definition()
                                }
                                Expression::Identifier(identifier) => {
                                    identifier.resolve_to_definition()
                                }
                                _ => None,
                            };
                            let builder = &context.state.builder;
                            let (selector_value, parameter_types, current) = match definition {
                                Some(Definition::Function(function)) => {
                                    let selector =
                                        function.compute_selector().expect("slang validated");
                                    let selector_value = AstValue::selector_constant(
                                        &BigInt::from(selector),
                                        4,
                                        &context.state.builder,
                                        &block,
                                    )
                                    .into_mlir();
                                    let (parameter_types, _) = AstType::resolve_signature(
                                        &function,
                                        LocationPolicy::ForceMemory,
                                        builder,
                                    );
                                    (selector_value, parameter_types, block)
                                }
                                _ => {
                                    let BlockAnd {
                                        value: function_value,
                                        block: current,
                                    } = function_expression.emit(context, block);
                                    let selector_value = function_value
                                        .ext_func_selector(builder, &current)
                                        .into_mlir();
                                    let SlangType::Function(function_type) =
                                        function_expression.get_type().expect("slang validated")
                                    else {
                                        unreachable!(
                                            "a non-static abi.encodeCall callee is a function pointer"
                                        )
                                    };
                                    let parameter_types = function_type
                                        .parameter_types()
                                        .iter()
                                        .map(|parameter_type| {
                                            AstType::resolve(
                                                parameter_type,
                                                LocationPolicy::ForceMemory,
                                                builder,
                                            )
                                        })
                                        .collect();
                                    (selector_value, parameter_types, current)
                                }
                            };
                            let argument_expressions: Vec<Expression> = match call_arguments {
                                Expression::TupleExpression(tuple) => tuple
                                    .items()
                                    .iter()
                                    .filter_map(|item| item.expression())
                                    .collect(),
                                other => vec![other],
                            };
                            let BlockAnd {
                                value: values,
                                block: current,
                            } = argument_expressions.emit_as(&parameter_types, context, current);
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
                        _ => unreachable!(
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

            // A member call `x.f(...)`, classified by operand and member resolution.
            let operand = access.operand();
            // `super.f` / a recorded base redirect: an internal call up the C3 chain.
            if matches!(operand, Expression::SuperKeyword(_))
                || context.state.super_redirect.contains_key(&access.node_id())
            {
                let target_id = context
                    .state
                    .super_redirect
                    .get(&access.node_id())
                    .copied()
                    .expect("a super/base call has a recorded redirect target");
                // Named arguments bind to the statically-resolved callee's parameter
                // names; ordering collapses positional and named forms into one path.
                let parameter_ids: Vec<NodeId> = match access.member().resolve_to_definition() {
                    Some(Definition::Function(function_definition)) => function_definition
                        .parameters()
                        .iter()
                        .map(|parameter| parameter.node_id())
                        .collect(),
                    _ => unreachable!("a super/base call resolves its member to a function"),
                };
                let argument_expressions = arguments.ordered_by(&parameter_ids).expect("slang validated");
                let function = context.state.resolve_function(target_id);
                let BlockAnd {
                    value: argument_values,
                    block,
                } = argument_expressions.emit_as(&function.parameter_types, context, block);
                let results = function.call(&argument_values, &context.state.builder, &block);
                return BlockAnd {
                    value: results,
                    block,
                };
            }

            let member_definition = access.member().resolve_to_definition();
            // An external library call (`L.f` namespace or `using for` onto a
            // selector-bearing library function) delegatecalls — the only member
            // call that accepts named arguments.
            if let Some(Definition::Function(function)) = &member_definition
                && function.compute_selector().is_some()
                && (matches!(&operand, Expression::Identifier(identifier)
                        if matches!(identifier.resolve_to_definition(), Some(Definition::Library(_))))
                    || matches!(
                        function.enclosing_definition(),
                        Some(Definition::Library(_))
                    ))
            {
                // Resolve the link target from the member-access callee: the
                // library function, its `solx_utils::ContractName`, and the `self`
                // receiver (`None` for a namespace-qualified `L.f`, the operand value
                // for a `using for` `x.f`).
                let Some(Definition::Library(library)) = function.enclosing_definition() else {
                    unreachable!("an external library call's target is a library member");
                };
                let library_operand = access.operand();
                let self_receiver = (!MemberAccessOperand(&library_operand)
                    .is_namespace_qualifier())
                .then_some(library_operand);
                let library_name = solx_utils::ContractName::new(
                    library.get_file_id().to_owned(),
                    Some(library.name().name()),
                );
                let parameter_ids: Vec<NodeId> = function
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let explicit_parameter_ids = if self_receiver.is_some() {
                    &parameter_ids[1..]
                } else {
                    &parameter_ids[..]
                };
                let argument_expressions = arguments.ordered_by(explicit_parameter_ids).expect("slang validated");

                // An external library call delegatecalls into the deployed library via `sol.ext_call`
                // (`delegate_call` + `library_call`). The address is a `sol.lib_addr` link placeholder;
                // a `using for` receiver becomes the implicit leading `self` argument.
                let (parameter_types, _) = AstType::resolve_signature(
                    function,
                    LocationPolicy::Declared(None),
                    &context.state.builder,
                );
                // A `calldata` return cannot cross the (delegate)call boundary, so it is decoded into
                // memory like every other external-call return. Only a `calldata` return is remapped:
                // a public library function may return a `storage` reference (e.g. a struct holding a
                // mapping), which is passed by slot and must keep its `storage` location; `memory` and
                // value returns are likewise kept as declared. Parameters keep their declared location
                // too — a `storage` reference parameter is passed by slot, not copied to memory.
                let return_types: Vec<_> = match function.returns() {
                    Some(returns) => returns
                        .iter()
                        .map(|parameter| {
                            let policy = if matches!(
                                parameter.storage_location(),
                                Some(slang_solidity_v2::ast::StorageLocation::CallDataKeyword(_))
                            ) {
                                LocationPolicy::ForceMemory
                            } else {
                                LocationPolicy::Declared(None)
                            };
                            AstType::resolve(
                                &parameter.get_type().expect("slang validated"),
                                policy,
                                &context.state.builder,
                            )
                        })
                        .collect(),
                    None => Vec::new(),
                };
                // A library external function keeps the ` storage` data-location in its selector
                // (e.g. `g(uint256[] storage)`), matching solc — see `library_aware_selector`.
                let selector = crate::ast::contract::function::signature::library_aware_selector(
                    function,
                )
                .expect("slang validated");
                let mlir_name = function.mlir_function_name();
                let (argument_values, current_block) = match &self_receiver {
                    Some(receiver) => {
                        let (parameter_self, parameter_rest) =
                            parameter_types.split_first().expect("slang validated");
                        let BlockAnd {
                            value: self_value,
                            block,
                        } = receiver.emit(context, block);
                        let builder = &context.state.builder;
                        let self_value = self_value
                            .cast(AstType::new(*parameter_self), builder, &block)
                            .into_mlir();
                        let BlockAnd {
                            value: mut rest_values,
                            block,
                        } = argument_expressions.emit_as(parameter_rest, context, block);
                        rest_values.insert(0, self_value);
                        (rest_values, block)
                    }
                    None => {
                        let BlockAnd { value, block } =
                            argument_expressions.emit_as(&parameter_types, context, block);
                        (value, block)
                    }
                };
                let builder = &context.state.builder;
                let address =
                    AstValue::library_address(&library_name, builder, &current_block).into_mlir();
                let callee_type =
                    FunctionType::new(builder.context, &parameter_types, &return_types);
                let gas = AstValue::gas_left(builder, &current_block).into_mlir();
                let value = AstValue::uint256(0, builder, &current_block).into_mlir();
                let selector_value =
                    AstValue::uint256(i64::from(selector), builder, &current_block).into_mlir();
                // `sol.ext_call` yields the `i1` status (result 0) then the decoded outs; it reverts
                // internally on failure, so the status is dropped and only the decoded results return.
                let operation = current_block.append_operation(mlir_op_build!(
                    builder,
                    ExtCallOperation
                        .callee(StringAttribute::new(builder.context, &mlir_name))
                        .ins(&argument_values)
                        .addr(address)
                        .gas(gas)
                        .val(value)
                        .selector(selector_value)
                        .delegate_call(Attribute::unit(builder.context))
                        .library_call(Attribute::unit(builder.context))
                        .callee_type(TypeAttribute::new(callee_type.into()))
                        .status(AstType::signless(
                            builder.context,
                            solx_utils::BIT_LENGTH_BOOLEAN
                        ))
                        .outs(&return_types)
                ));
                let results = (0..return_types.len())
                    .map(|index| {
                        operation
                            .result(index + 1)
                            .expect("sol.ext_call produces the declared results")
                            .into()
                    })
                    .collect();
                return BlockAnd {
                    value: results,
                    block: current_block,
                };
            }

            // A member call orders named arguments against the callee's parameters
            // (a `using for` receiver is the implicit `self`, a getter takes positional
            // key/index arguments only — see the arms below).
            return match member_definition {
                // `using for` / `L.f` onto an internal (no-selector) library fn,
                // inlined like an ordinary internal call; a selector-bearing one is
                // a `this.f` / `instance.f` external call.
                Some(Definition::Function(function)) if function.compute_selector().is_none() => {
                    let resolved = context.state.resolve_function(function.node_id());
                    let parameter_ids: Vec<NodeId> = function
                        .parameters()
                        .iter()
                        .map(|parameter| parameter.node_id())
                        .collect();
                    // A namespace qualifier (`L.f` / `M.f`) is not a value, so only
                    // the explicit arguments pass; a `using for` receiver becomes the
                    // implicit `self` first parameter.
                    if MemberAccessOperand(&operand).is_namespace_qualifier() {
                        let argument_expressions = arguments.ordered_by(&parameter_ids).expect("slang validated");
                        let BlockAnd {
                            value: argument_values,
                            block,
                        } = argument_expressions.emit_as(&resolved.parameter_types, context, block);
                        let results =
                            resolved.call(&argument_values, &context.state.builder, &block);
                        BlockAnd {
                            value: results,
                            block,
                        }
                    } else {
                        let (parameter_self, parameter_rest) = resolved
                            .parameter_types
                            .split_first()
                            .expect("slang validated");
                        let BlockAnd {
                            value: self_value,
                            block,
                        } = operand.emit(context, block);
                        let self_value = self_value
                            .cast(
                                AstType::new(*parameter_self),
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir();
                        // The receiver is the implicit `self`; order only the explicit
                        // arguments against the remaining parameter ids.
                        let argument_expressions = arguments.ordered_by(&parameter_ids[1..]).expect("slang validated");
                        let BlockAnd {
                            value: mut argument_values,
                            block,
                        } = argument_expressions.emit_as(parameter_rest, context, block);
                        argument_values.insert(0, self_value);
                        let results =
                            resolved.call(&argument_values, &context.state.builder, &block);
                        BlockAnd {
                            value: results,
                            block,
                        }
                    }
                }
                // `this.f` / `instance.f` (external call) and `this.x` / `instance.x` (getter) converge
                // on one `sol.ext_icall`, differing only in the selector and signature source. A
                // `view`/`pure` callee lowers to a STATICCALL. A nested / reference-typed getter is a LOUD residual.
                Some(Definition::Function(_) | Definition::StateVariable(_)) => {
                    // An external function call orders named arguments against the
                    // callee's parameters; a getter takes positional key/index
                    // arguments only (its synthetic ABI signature has no names).
                    let ordered_arguments: Vec<Expression> =
                        match access.member().resolve_to_definition() {
                            Some(Definition::Function(function)) => {
                                let parameter_ids: Vec<NodeId> = function
                                    .parameters()
                                    .iter()
                                    .map(|parameter| parameter.node_id())
                                    .collect();
                                arguments.ordered_by(&parameter_ids).expect("slang validated")
                            }
                            _ => {
                                let ArgumentsDeclaration::PositionalArguments(positional) =
                                    &arguments
                                else {
                                    unreachable!("named arguments on a getter are not supported");
                                };
                                positional.iter().collect()
                            }
                        };
                    let (selector, parameter_types, return_types, is_static) = match access
                        .member()
                        .resolve_to_definition()
                    {
                        Some(Definition::Function(function)) => {
                            let (parameter_types, return_types) = AstType::resolve_signature(
                                &function,
                                LocationPolicy::ForceMemory,
                                &context.state.builder,
                            );
                            (
                                function.compute_selector().expect("slang validated"),
                                parameter_types,
                                return_types,
                                matches!(
                                    function.mutability(),
                                    FunctionMutability::View | FunctionMutability::Pure
                                ),
                            )
                        }
                        Some(Definition::StateVariable(state_variable)) => {
                            // A `this.m(key)` self getter and an `other.m(key)` getter on another
                            // instance share one path: the receiver is emitted below and the
                            // key/index arguments are passed against the getter's signature.
                            // The getter's external ABI signature (scalar / mapping / array / struct,
                            // multi-level). `None` only for a struct with no returnable members, which
                            // slang rejects (solc: "the struct has all its members omitted").
                            let builder = &context.state.builder;
                            let Some((parameter_types, return_types)) =
                                state_variable.getter_signature(builder)
                            else {
                                unreachable!(
                                    "slang rejects a getter on a struct with no returnable members"
                                );
                            };
                            (
                                state_variable.compute_selector().expect("slang validated"),
                                parameter_types,
                                return_types,
                                false,
                            )
                        }
                        _ => unreachable!(
                            "an external member call resolves to a function or state variable"
                        ),
                    };
                    let BlockAnd {
                        value: receiver,
                        block,
                    } = access.operand().emit(context, block);
                    let BlockAnd {
                        value: argument_values,
                        block,
                    } = ordered_arguments.emit_as(&parameter_types, context, block);
                    let builder = &context.state.builder;
                    let callee = AstValue::external_callee(
                        receiver,
                        selector,
                        &parameter_types,
                        &return_types,
                        builder,
                        &block,
                    );
                    // A `view`/`pure` callee lowers to a STATICCALL; `{value: v}`
                    // forwards `v`, `{gas: g}` caps the forwarded gas, a plain call sends zero and
                    // forwards all gas — all handled by `call_indirect` (the callee is an `ext_func_ref`).
                    let results = callee.call_indirect(
                        &argument_values,
                        &return_types,
                        call_value,
                        call_gas,
                        is_static,
                        builder,
                        &block,
                    );
                    BlockAnd {
                        value: results,
                        block,
                    }
                }
                other => unreachable!(
                    "unsupported member call: {:?}",
                    other.map(|definition| definition.node_id())
                ),
            };
        }

        // `new T[](n)` / `new bytes(n)` / `new C(args)`.
        if let Expression::NewExpression(_) = &callee {
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
                // `new T[](n)` / `new bytes(n)` / `new string(n)` take a single positional
                // size argument; solc rejects named arguments on these forms.
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unreachable!("named arguments on a new array/bytes/string are not supported");
                };
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
                unreachable!("new expression has no resolved type or unsupported new target");
            };
            let Definition::Contract(contract_definition) = contract_type.definition() else {
                unreachable!("Slang ContractType always references a Contract definition");
            };
            // Named constructor arguments are ordered against the constructor's
            // parameters; positional arguments pass through unchanged.
            let parameter_ids: Vec<NodeId> = contract_definition
                .constructor()
                .map(|constructor| {
                    constructor
                        .parameters()
                        .iter()
                        .map(|parameter| parameter.node_id())
                        .collect()
                })
                .unwrap_or_default();
            let ordered_arguments = arguments.ordered_by(&parameter_ids).expect("slang validated");
            let (value, block) = contract_definition.emit_creation(
                context,
                ordered_arguments,
                call_value,
                salt,
                false,
                block,
            );
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
                && !array_type.is_slice()
            {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unreachable!("named arguments on an array-type cast are not supported");
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
            unreachable!("unsupported callee expression");
        };
        match identifier.resolve_to_definition() {
            // A direct call passes its arguments by position or by name; ordering
            // them against the parameter ids collapses both into one path.
            Some(Definition::Function(function_definition)) => {
                let parameter_ids: Vec<NodeId> = function_definition
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let ordered = arguments.ordered_by(&parameter_ids).expect("slang validated");
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
            _ => unreachable!(
                "callee '{}' does not resolve to a function",
                identifier.name()
            ),
        }
    }
}

/// Emits a `new C(args)` contract creation as `sol.new`, recording the link dependency so the linker
/// pulls the deploy object in. With `try_call`, marks the creation (`sol.new ... try`) so a surrounding
/// `sol.try` receives a success status instead of reverting. Shared by the plain `new` path and the
/// `try new` path (`TryNewExpression`) so both emit identical sol-dialect.
/// Emits the creation of a contract instance — shared by the `new C(args)`
/// expression path and the `try new C(args)` path — lowering to `sol.new` and
/// recording the linker dependency on the created contract.
pub(crate) trait ContractCreation {
    /// Emits `new self(ordered_arguments)`, forwarding `{value}`/`{salt}` and using
    /// `try` semantics when `try_call` is set. `ordered_arguments` are already in the
    /// constructor's declaration order (see [`ArgumentsDeclaration::ordered_by`]).
    fn emit_creation<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        ordered_arguments: Vec<Expression>,
        call_value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
        try_call: bool,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>);
}

impl ContractCreation for slang_solidity_v2::ast::ContractDefinition {
    fn emit_creation<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        ordered_arguments: Vec<Expression>,
        call_value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
        try_call: bool,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let contract_name = self.name().name();
        let payable = self.is_payable();
        context.state.add_dependency(contract_name.clone());

        // Coerce each constructor argument to its declared parameter type so a literal materialises in the
        // parameter's representation (the deployed constructor ABI-decodes by parameter type).
        let parameter_types = self
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
        let BlockAnd {
            value: ctor_args,
            block,
        } = ordered_arguments.emit_as(&parameter_types, context, block);
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
            try_call,
            builder,
            &block,
        )
        .into_mlir();
        (value, block)
    }
}
