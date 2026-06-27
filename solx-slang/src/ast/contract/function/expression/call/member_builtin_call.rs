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
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_mlir::ods::sol::ConcatOperation;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::PopOperation;
use solx_mlir::ods::sol::PushStringOperation;
use solx_mlir::ods::sol::SendOperation;
use solx_mlir::ods::sol::TransferOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

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
                let builder = &context.state.builder;
                let input = input
                    .cast(
                        AstType::string(builder.context, solx_utils::DataLocation::Memory),
                        builder,
                        &block,
                    )
                    .into_mlir();
                let address = address.into_mlir();
                let status_type =
                    AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
                let ret_data_type =
                    AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
                let operation = match kind {
                    BuiltIn::AddressCall => {
                        let value = call_value
                            .unwrap_or_else(|| AstValue::uint256(0, builder, &block).into_mlir());
                        mlir_op_build!(
                            builder,
                            BareCallOperation
                                .addr(address)
                                .gas(
                                    call_gas
                                        .map(AstValue::from)
                                        .unwrap_or_else(|| AstValue::gas_left(builder, &block))
                                )
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
                            .gas(
                                call_gas
                                    .map(AstValue::from)
                                    .unwrap_or_else(|| AstValue::gas_left(builder, &block))
                            )
                            .inp(input)
                            .status(status_type)
                            .ret_data(ret_data_type)
                    ),
                    BuiltIn::AddressStaticcall => mlir_op_build!(
                        builder,
                        BareStaticCallOperation
                            .addr(address)
                            .gas(
                                call_gas
                                    .map(AstValue::from)
                                    .unwrap_or_else(|| AstValue::gas_left(builder, &block))
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
                return BlockAnd {
                    value: vec![status, ret_data],
                    block,
                };
            }
            BuiltIn::AbiDecode => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!("abi.decode takes positional arguments only; named arguments are invalid");
                };
                let payload_expression = positional.iter().next().expect("slang validated");
                let BlockAnd {
                    value: payload_value,
                    block,
                } = payload_expression.emit(context, block);
                let result_types: Vec<Type> = AstType::resolve_result_types(
                    &self.call.get_type().expect("slang validated"),
                    &context.state.builder,
                );
                let builder = &context.state.builder;
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
            BuiltIn::Wrap | BuiltIn::Unwrap => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!("a UDVT wrap/unwrap takes one positional argument");
                };
                let argument = positional.iter().next().expect("slang validated");
                let BlockAnd { value, block } = argument.emit(context, block);
                let result =
                    match AstType::resolve_optional(self.call.get_type(), &context.state.builder) {
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
            member_built_in => {
                if matches!(member_built_in, BuiltIn::ArrayPop | BuiltIn::ArrayPush)
                    && matches!(&self.arguments,
                        ArgumentsDeclaration::NamedArguments(named) if named.iter().next().is_none())
                {
                    let base_slang_type =
                        self.access.operand().get_type().expect("slang validated");
                    let BlockAnd {
                        value: array_value,
                        block,
                    } = self.access.operand().emit(context, block);
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
                let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                    unreachable!(
                        "named arguments on a member built-in are invalid (other than empty `pop`/`push` braces)"
                    );
                };
                let (value, block) = match member_built_in {
                    BuiltIn::AddressSend => {
                        let builder = &context.state.builder;
                        let BlockAnd { value: addr, block } =
                            self.access.operand().emit(context, block);
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let amount = AstValue::from(values[0])
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
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
                        let builder = &context.state.builder;
                        let BlockAnd { value: addr, block } =
                            self.access.operand().emit(context, block);
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let amount = AstValue::from(values[0])
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &block,
                            )
                            .into_mlir();
                        mlir_op_void!(builder, block, TransferOperation.addr(addr).val(amount));
                        (None, block)
                    }
                    BuiltIn::AbiEncode => {
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let builder = &context.state.builder;
                        let result =
                            AstValue::abi_encode(&values, None, false, builder, &block).into_mlir();
                        (Some(result), block)
                    }
                    BuiltIn::AbiEncodePacked => {
                        let BlockAnd {
                            value: values,
                            block,
                        } = positional.emit(context, block);
                        let builder = &context.state.builder;
                        let result =
                            AstValue::abi_encode(&values, None, true, builder, &block).into_mlir();
                        (Some(result), block)
                    }
                    BuiltIn::AbiEncodeWithSelector => {
                        let BlockAnd {
                            value: mut values,
                            block,
                        } = positional.emit(context, block);
                        let builder = &context.state.builder;
                        let selector = AstValue::from(values.remove(0))
                            .cast(AstType::fixed_bytes(builder.context, 4), builder, &block)
                            .into_mlir();
                        let result =
                            AstValue::abi_encode(&values, Some(selector), false, builder, &block)
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
                            let BlockAnd { value, block: next } = argument.emit(context, current);
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
                        let BlockAnd {
                            value: array_value,
                            block,
                        } = self.access.operand().emit(context, block);
                        mlir_op_void!(
                            &context.state.builder,
                            &block,
                            PopOperation.inp(array_value)
                        );
                        (None, block)
                    }
                    BuiltIn::ArrayPush => {
                        let base = self.access.operand();
                        let base_slang_type = base.get_type().expect("slang validated");
                        let value_argument = positional.iter().next();
                        if let (SlangType::Bytes(_), Some(value_argument)) =
                            (&base_slang_type, &value_argument)
                        {
                            let BlockAnd {
                                value: bytes_reference,
                                block,
                            } = base.emit(context, block);
                            let byte_target =
                                AstType::fixed_bytes(context.state.builder.context, 1).into_mlir();
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
                                self.access.operand().get_type().expect("slang validated");
                            let BlockAnd {
                                value: array_value,
                                block,
                            } = self.access.operand().emit(context, block);
                            let (new_slot, element_type) = array_value.push_slot(
                                &base_slang_type,
                                &context.state.builder,
                                &block,
                            );
                            let new_slot = new_slot.into_mlir();
                            let Some(value_argument) = value_argument else {
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
                                let BlockAnd { value, block } = value_argument.emit(context, block);
                                Pointer::new(new_slot).copy_from(
                                    value,
                                    &context.state.builder,
                                    &block,
                                );
                                (None, block)
                            } else {
                                let BlockAnd { value, block } =
                                    value_argument.emit_as(element_type, context, block);
                                Pointer::new(new_slot).store(value, &context.state.builder, &block);
                                (None, block)
                            }
                        }
                    }
                    BuiltIn::StringConcat | BuiltIn::BytesConcat => {
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
