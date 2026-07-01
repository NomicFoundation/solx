//!
//! Member-position Solidity built-in calls.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use num_bigint::BigInt;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::LocationPolicy;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::PopOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

/// A Solidity built-in called through member access.
pub struct MemberBuiltinCall {
    /// The full call expression.
    pub call: FunctionCallExpression,
    /// The member access that resolved to the built-in.
    pub access: MemberAccessExpression,
    /// The resolved built-in.
    pub built_in: BuiltIn,
    /// The call arguments.
    pub arguments: ArgumentsDeclaration,
}

impl MemberBuiltinCall {
    /// Classifies a member-position built-in call.
    pub fn from_call(call: &FunctionCallExpression, callee: &Expression) -> Option<Self> {
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        let built_in = access.member().resolve_to_built_in()?;
        Some(Self {
            call: call.clone(),
            access: access.clone(),
            built_in,
            arguments: call.arguments(),
        })
    }

    /// Emits the built-in call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        match &self.built_in {
            kind @ (BuiltIn::AddressCall
            | BuiltIn::AddressDelegatecall
            | BuiltIn::AddressStaticcall) => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!("a bare low-level call takes positional arguments only");
                };
                let BlockAnd {
                    value: address,
                    block,
                } = self.access.operand().emit(context, block);
                let argument = positional.iter().next().expect("slang validated");
                let BlockAnd {
                    value: input,
                    block,
                } = argument.emit(context, block);
                let state = context.state;
                let input = input
                    .cast(
                        AstType::string(state.mlir_context, solx_utils::DataLocation::Memory),
                        state,
                        &block,
                    )
                    .into_mlir();
                let address = address.into_mlir();
                let status_type =
                    AstType::signless(state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN)
                        .into_mlir();
                let ret_data_type =
                    AstType::string(state.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir();
                let operation = match kind {
                    BuiltIn::AddressCall => {
                        let value = call_value
                            .unwrap_or_else(|| AstValue::uint256(0, state, &block).into_mlir());
                        mlir_op_build!(
                            state,
                            BareCallOperation
                                .addr(address)
                                .gas(
                                    call_gas
                                        .map(AstValue::from)
                                        .unwrap_or_else(|| AstValue::gas_left(state, &block))
                                )
                                .val(value)
                                .inp(input)
                                .status(status_type)
                                .ret_data(ret_data_type)
                        )
                    }
                    BuiltIn::AddressDelegatecall => mlir_op_build!(
                        state,
                        BareDelegateCallOperation
                            .addr(address)
                            .gas(
                                call_gas
                                    .map(AstValue::from)
                                    .unwrap_or_else(|| AstValue::gas_left(state, &block))
                            )
                            .inp(input)
                            .status(status_type)
                            .ret_data(ret_data_type)
                    ),
                    BuiltIn::AddressStaticcall => mlir_op_build!(
                        state,
                        BareStaticCallOperation
                            .addr(address)
                            .gas(
                                call_gas
                                    .map(AstValue::from)
                                    .unwrap_or_else(|| AstValue::gas_left(state, &block))
                            )
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
                BlockAnd {
                    value: vec![status, ret_data],
                    block,
                }
            }
            BuiltIn::AbiDecode => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!(
                        "abi.decode takes positional arguments only; named arguments are invalid"
                    );
                };
                let payload_expression = positional.iter().next().expect("slang validated");
                let BlockAnd {
                    value: payload_value,
                    block,
                } = payload_expression.emit(context, block);
                let result_types: Vec<Type> = AstType::resolve_result_types(
                    &self.call.get_type().expect("slang validated"),
                    context.state,
                );
                let state = context.state;
                let payload_value = if matches!(
                    payload_expression
                        .get_type()
                        .and_then(|payload_type| payload_type.data_location()),
                    Some(SlangDataLocation::Storage)
                ) {
                    payload_value.cast(
                        AstType::string(state.mlir_context, solx_utils::DataLocation::Memory),
                        state,
                        &block,
                    )
                } else {
                    payload_value
                };
                let operation = block.append_operation(mlir_op_build!(
                    state,
                    DecodeOperation
                        .addr(payload_value.into_mlir())
                        .outs(result_types.as_slice())
                ));
                let values = (0..result_types.len())
                    .map(|index| {
                        operation
                            .result(index)
                            .expect("sol.decode yields one result per requested type")
                            .into()
                    })
                    .collect();
                BlockAnd {
                    value: values,
                    block,
                }
            }
            BuiltIn::Wrap | BuiltIn::Unwrap => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!("a UDVT wrap/unwrap takes one positional argument");
                };
                let argument = positional.iter().next().expect("slang validated");
                let BlockAnd { value, block } = argument.emit(context, block);
                let result = match AstType::resolve_optional(self.call.get_type(), context.state) {
                    Some(result_type) => value
                        .cast(AstType::new(result_type), context.state, &block)
                        .into_mlir(),
                    None => value.into_mlir(),
                };
                BlockAnd {
                    value: vec![result],
                    block,
                }
            }
            member_built_in => {
                if matches!(member_built_in, BuiltIn::ArrayPop | BuiltIn::ArrayPush) {
                    let value_argument = match &self.arguments {
                        ArgumentsDeclaration::PositionalArguments(positional) => {
                            positional.iter().next()
                        }
                        ArgumentsDeclaration::NamedArguments(named)
                            if named.iter().next().is_none() =>
                        {
                            None
                        }
                        _ => unreachable!("array push/pop takes at most one positional argument"),
                    };
                    let base = self.access.operand();
                    let base_slang_type = base.get_type().expect("slang validated");
                    if let BuiltIn::ArrayPop = member_built_in {
                        let BlockAnd {
                            value: array_value,
                            block,
                        } = base.emit(context, block);
                        mlir_op_void!(context.state, &block, PopOperation.inp(array_value));
                        return BlockAnd {
                            value: vec![],
                            block,
                        };
                    }
                    if let (SlangType::Bytes(_), Some(value_argument)) =
                        (&base_slang_type, &value_argument)
                    {
                        let BlockAnd {
                            value: bytes_reference,
                            block,
                        } = base.emit(context, block);
                        let byte_target =
                            AstType::fixed_bytes(context.state.mlir_context, 1).into_mlir();
                        let BlockAnd { value, block } =
                            value_argument.emit_as(byte_target, context, block);
                        let state = context.state;
                        let byte_value = value.into_mlir();
                        mlir_op_void!(
                            state,
                            &block,
                            PushStringOperation.addr(bytes_reference).value(byte_value)
                        );
                        return BlockAnd {
                            value: vec![],
                            block,
                        };
                    }
                    let BlockAnd {
                        value: array_value,
                        block,
                    } = base.emit(context, block);
                    let (new_slot, element_type) =
                        array_value.push_slot(&base_slang_type, context.state, &block);
                    let new_slot = new_slot.into_mlir();
                    let Some(value_argument) = value_argument else {
                        let loaded = Pointer::new(new_slot)
                            .load(AstType::new(element_type), context.state, &block)
                            .into_mlir();
                        return BlockAnd {
                            value: vec![loaded],
                            block,
                        };
                    };
                    if AstType::new(element_type).is_reference() {
                        let BlockAnd { value, block } = value_argument.emit(context, block);
                        Pointer::new(new_slot).copy_from(value, context.state, &block);
                    } else {
                        let BlockAnd { value, block } =
                            value_argument.emit_as(element_type, context, block);
                        Pointer::new(new_slot).store(value, context.state, &block);
                    }
                    return BlockAnd {
                        value: vec![],
                        block,
                    };
                }
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!("named arguments on a member built-in are invalid");
                };
                let (value, block) = match member_built_in {
                    BuiltIn::AddressSend => {
                        let state = context.state;
                        let BlockAnd {
                            value: address,
                            block,
                        } = self.access.operand().emit(context, block);
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let amount = AstValue::from(values[0])
                            .cast(
                                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                                state,
                                &block,
                            )
                            .into_mlir();
                        let value = mlir_op!(
                            state,
                            block,
                            SendOperation
                                .addr(address)
                                .val(amount)
                                .status(AstType::signless(
                                    state.mlir_context,
                                    solx_utils::BIT_LENGTH_BOOLEAN
                                ))
                        );
                        (Some(value), block)
                    }
                    BuiltIn::AddressTransfer => {
                        let state = context.state;
                        let BlockAnd {
                            value: address,
                            block,
                        } = self.access.operand().emit(context, block);
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let amount = AstValue::from(values[0])
                            .cast(
                                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                                state,
                                &block,
                            )
                            .into_mlir();
                        mlir_op_void!(state, block, TransferOperation.addr(address).val(amount));
                        (None, block)
                    }
                    BuiltIn::AbiEncode => {
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let state = context.state;
                        let result =
                            AstValue::abi_encode(&values, None, false, state, &block).into_mlir();
                        (Some(result), block)
                    }
                    BuiltIn::AbiEncodePacked => {
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let state = context.state;
                        let result =
                            AstValue::abi_encode(&values, None, true, state, &block).into_mlir();
                        (Some(result), block)
                    }
                    BuiltIn::AbiEncodeWithSelector => {
                        let BlockAnd {
                            value: mut values,
                            block,
                        } = positional.emit(context, block);
                        let state = context.state;
                        let selector = AstValue::from(values.remove(0))
                            .cast(AstType::fixed_bytes(state.mlir_context, 4), state, &block)
                            .into_mlir();
                        let result =
                            AstValue::abi_encode(&values, Some(selector), false, state, &block)
                                .into_mlir();
                        (Some(result), block)
                    }
                    BuiltIn::AbiEncodeWithSignature => {
                        let mut iter = positional.iter();
                        let signature_expression = iter.next().expect("slang validated");
                        let (selector_value, mut current) = match &signature_expression {
                            Expression::StringExpression(string_expression) => {
                                let signature_bytes = string_expression.value();
                                let hash = solx_utils::Keccak256Hash::from_slice(&signature_bytes);
                                let selector_bytes: [u8; 4] = hash.as_bytes()[..4]
                                    .try_into()
                                    .expect("keccak256 always yields 32 bytes");
                                let selector_word = u32::from_be_bytes(selector_bytes);
                                let selector_value = AstValue::selector_constant(
                                    &BigInt::from(selector_word),
                                    4,
                                    context.state,
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
                                let hash =
                                    AstValue::keccak256(signature_value, context.state, &current);
                                let state = context.state;
                                let selector_value = hash
                                    .cast(
                                        AstType::fixed_bytes(state.mlir_context, 4),
                                        state,
                                        &current,
                                    )
                                    .into_mlir();
                                (selector_value, current)
                            }
                        };
                        let mut values = Vec::with_capacity(positional.len() - 1);
                        for argument in iter {
                            let BlockAnd { value, block: next } = argument.emit(context, current);
                            values.push(value.into_mlir());
                            current = next;
                        }
                        let state = context.state;
                        let result = AstValue::abi_encode(
                            &values,
                            Some(selector_value),
                            false,
                            state,
                            &current,
                        )
                        .into_mlir();
                        (Some(result), current)
                    }
                    BuiltIn::AbiEncodeCall => {
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
                        let state = context.state;
                        let (selector_value, parameter_types, current) = match definition {
                            Some(Definition::Function(function)) => {
                                let selector =
                                    function.compute_selector().expect("slang validated");
                                let selector_value = AstValue::selector_constant(
                                    &BigInt::from(selector),
                                    4,
                                    context.state,
                                    &block,
                                )
                                .into_mlir();
                                let (parameter_types, _) = AstType::resolve_signature(
                                    &function,
                                    LocationPolicy::ForceMemory,
                                    state,
                                );
                                (selector_value, parameter_types, block)
                            }
                            _ => {
                                let BlockAnd {
                                    value: function_value,
                                    block: current,
                                } = function_expression.emit(context, block);
                                let selector_value = function_value
                                    .ext_func_selector(state, &current)
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
                                            state,
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
                            state,
                            &current,
                        )
                        .into_mlir();
                        (Some(result), current)
                    }
                    BuiltIn::StringConcat | BuiltIn::BytesConcat => {
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let state = context.state;
                        let result_type =
                            AstType::string(state.mlir_context, solx_utils::DataLocation::Memory)
                                .into_mlir();
                        let value = mlir_op!(
                            state,
                            block,
                            ConcatOperation.args(&values).result(result_type)
                        );
                        (Some(value), block)
                    }
                    _ => unreachable!(
                        "unsupported call-position member built-in: {}",
                        self.access.member().name()
                    ),
                };
                BlockAnd {
                    value: value.into_iter().collect(),
                    block,
                }
            }
        }
    }
}
