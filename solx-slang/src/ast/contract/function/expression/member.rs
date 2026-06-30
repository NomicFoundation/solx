//!
//! Member access expression emission for `base.member`: namespace-qualified reads, struct fields, and built-ins.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_mlir::ods::sol::BalanceOperation;
use solx_mlir::ods::sol::BaseFeeOperation;
use solx_mlir::ods::sol::BlobBaseFeeOperation;
use solx_mlir::ods::sol::BlockNumberOperation;
use solx_mlir::ods::sol::CallValueOperation;
use solx_mlir::ods::sol::CallerOperation;
use solx_mlir::ods::sol::ChainIdOperation;
use solx_mlir::ods::sol::CodeHashOperation;
use solx_mlir::ods::sol::CodeOperation;
use solx_mlir::ods::sol::CoinbaseOperation;
use solx_mlir::ods::sol::DifficultyOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::ObjectCodeOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::TimestampOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::EmitPlace;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::analysis::query::MemberAccessOperand;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::getter::signature::Signature;

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for MemberAccessExpression {
    /// Emits the address `s.field` denotes with the field's element type, without the load. Struct base only.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        let base = self.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            unreachable!("a struct-field address is only emitted for a struct base");
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        let Some(Definition::StructMember(member_definition)) =
            self.member().resolve_to_definition()
        else {
            unreachable!("slang resolves a struct field access to its StructMember definition");
        };
        let member_id = member_definition.node_id();
        let field_index = struct_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_id)
            .expect("slang validated");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block);
        let state = context.state;

        let index_value = AstValue::constant(
            field_index as i64,
            AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_X64),
            state,
            &block,
        );
        let element_type = base_value.r#type().element_type(field_index);
        let address = Pointer::from(base_value)
            .gep(index_value, element_type, false, state, &block)
            .into_mlir();
        BlockAnd {
            value: Place {
                address,
                element_type: element_type.into_mlir(),
            },
            block,
        }
    }
}

expression_emit!(MemberAccessExpression; |node, context, block| {
    if let Expression::Identifier(operand) = node.operand()
        && matches!(
            operand.resolve_to_definition(),
            Some(
                Definition::Contract(_)
                    | Definition::Library(_)
                    | Definition::Import(_)
                    | Definition::ImportedSymbol(_)
            )
        )
    {
        // A `Function` member is NOT delegated: it would pick up the virtual redirect `Base.f` must skip.
        match node.member().resolve_to_definition() {
            Some(Definition::StateVariable(_) | Definition::Constant(_)) => {
                return node.member().emit(context, block);
            }
            Some(Definition::Function(function_definition))
                if matches!(
                    operand.resolve_to_definition(),
                    Some(Definition::Contract(_))
                ) =>
            {
                let value = context
                    .state
                    .resolve_function(function_definition.node_id())
                    .pointer_constant(context.state, &block);
                return BlockAnd { block, value };
            }
            _ => {}
        }
    }
    let built_in = node.member().resolve_to_built_in();
    match &built_in {
        Some(builtin @ (BuiltIn::TypeMin | BuiltIn::TypeMax)) => {
            let result_type =
                AstType::resolve_optional(node.get_type(), context.state)
                    .expect("slang validated");
            let integer_type = IntegerType::try_from(result_type).expect("slang validated");
            let bits = AstType::new(result_type).integer_bit_width() as usize;
            let integer = match (builtin, integer_type.is_signed()) {
                (BuiltIn::TypeMin, false) => BigInt::ZERO,
                (BuiltIn::TypeMin, true) => -(BigInt::from(1) << (bits - 1)),
                (BuiltIn::TypeMax, false) => (BigInt::from(1) << bits) - 1,
                (BuiltIn::TypeMax, true) => (BigInt::from(1) << (bits - 1)) - 1,
                _ => unreachable!("dispatched on TypeMin / TypeMax"),
            };
            let value = AstValue::constant_from_bigint(
                &integer,
                AstType::new(result_type),
                context.state,
                &block,
            );
            return BlockAnd { block, value };
        }
        Some(builtin @ (BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax)) => {
            let Expression::TypeExpression(type_expression) = node.operand() else {
                unreachable!("type(E).min/max operand is a type expression");
            };
            let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            else {
                unreachable!("type(E) names an enum via an identifier path");
            };
            let Some(Definition::Enum(enum_definition)) =
                identifier_path.resolve_to_definition()
            else {
                unreachable!("type(E).min/max resolves to an enum definition");
            };
            let result_type =
                AstType::resolve_optional(node.get_type(), context.state)
                    .expect("slang validated");
            let member_count = enum_definition.members().iter().count();
            let ordinal = match builtin {
                BuiltIn::TypeEnumMin => 0,
                BuiltIn::TypeEnumMax => member_count.saturating_sub(1) as i64,
                _ => unreachable!("dispatched on TypeEnumMin / TypeEnumMax"),
            };
            let state = context.state;
            let value = AstValue::uint256(ordinal, state, &block)
                .cast(AstType::new(result_type), state, &block);
            return BlockAnd { block, value };
        }
        Some(BuiltIn::TypeInterfaceId) => {
            let Expression::TypeExpression(type_expression) = node.operand() else {
                unreachable!("type(I).interfaceId operand is a type expression");
            };
            let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            else {
                unreachable!("type(I) names an interface via an identifier path");
            };
            let Some(Definition::Interface(interface_definition)) =
                identifier_path.resolve_to_definition()
            else {
                unreachable!("type(I).interfaceId resolves to an interface definition");
            };
            let interface_id = interface_definition
                .members()
                .iter()
                .filter_map(|member| match member {
                    ContractMember::FunctionDefinition(function) => function.compute_selector(),
                    _ => None,
                })
                .fold(0u32, |interface_id, selector| interface_id ^ selector);
            let value =
                AstValue::selector_constant(&BigInt::from(interface_id), 4, context.state, &block);
            return BlockAnd { block, value };
        }
        Some(BuiltIn::TypeName) => {
            let Expression::TypeExpression(type_expression) = node.operand() else {
                unreachable!("type(C).name operand is a type expression");
            };
            let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            else {
                unreachable!("type(C) names a contract via an identifier path");
            };
            let type_name = match identifier_path.resolve_to_definition() {
                Some(Definition::Contract(contract)) => contract.name().name(),
                Some(Definition::Interface(interface)) => interface.name().name(),
                _ => unreachable!("type(C).name resolves to a contract or interface"),
            };
            let value = AstValue::string_literal(&type_name, context.state, &block);
            return BlockAnd { block, value };
        }
        Some(builtin @ (BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode)) => {
            let Expression::TypeExpression(type_expression) = node.operand() else {
                unreachable!("type(C).creationCode/runtimeCode operand is a type expression");
            };
            let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            else {
                unreachable!("type(C) names a contract via an identifier path");
            };
            let Some(Definition::Contract(contract_definition)) =
                identifier_path.resolve_to_definition()
            else {
                unreachable!("type(C).creationCode/runtimeCode resolves to a contract definition");
            };
            let contract_name = contract_definition.name().name();
            // `runtimeCode` must depend on the deployed object `C_deployed`; depending on the
            // creation object `C` alone leaves its data symbols unresolved at link time.
            let object_name = match builtin {
                BuiltIn::TypeRuntimeCode => {
                    format!("{contract_name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX)
                }
                _ => contract_name,
            };
            context.state.add_dependency(object_name.clone());
            let result_type = AstType::resolve_optional(node.get_type(), context.state)
                .expect("slang validated");
            let state = context.state;
            let value: MlirValue<'context, 'block> = mlir_op!(
                state,
                &block,
                ObjectCodeOperation
                    .obj_name(StringAttribute::new(state.mlir_context, &object_name))
                    .out(result_type)
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::AddressBalance) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let state = context.state;
            let value: MlirValue<'context, 'block> = mlir_op!(
                state,
                &block,
                BalanceOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::AddressCodehash) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let state = context.state;
            let value: MlirValue<'context, 'block> = mlir_op!(
                state,
                &block,
                CodeHashOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::AddressCode) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let state = context.state;
            let value: MlirValue<'context, 'block> = mlir_op!(
                state,
                &block,
                CodeOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::string(state.mlir_context, solx_utils::DataLocation::Memory))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(BuiltIn::Length) => {
            let BlockAnd {
                value: operand,
                block,
            } = node.operand().emit(context, block);
            return BlockAnd {
                value: operand.length(context.state, &block),
                block,
            };
        }
        Some(
            BuiltIn::AddressTransfer
            | BuiltIn::AddressSend
            | BuiltIn::AddressCall
            | BuiltIn::AddressDelegatecall
            | BuiltIn::AddressStaticcall
            | BuiltIn::ArrayPop
            | BuiltIn::ArrayPush
            | BuiltIn::AbiEncode
            | BuiltIn::AbiEncodePacked
            | BuiltIn::AbiEncodeWithSelector
            | BuiltIn::AbiEncodeWithSignature
            | BuiltIn::AbiEncodeCall
            | BuiltIn::AbiDecode
            | BuiltIn::Wrap
            | BuiltIn::Unwrap,
        ) => {
            unreachable!(
                "address/array/abi/wrap member builtins are only valid as a call callee, intercepted upstream"
            )
        }
        _ => {}
    }
    let state = context.state;
    let environment_op = match &built_in {
        Some(BuiltIn::TxOrigin) => {
            Some(mlir_op_build!(state, OriginOperation.addr(AstType::address(state.mlir_context, false))))
        }
        Some(BuiltIn::TxGasPrice) => Some(mlir_op_build!(
            state,
            GasPriceOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSender) => {
            Some(mlir_op_build!(state, CallerOperation.addr(AstType::address(state.mlir_context, false))))
        }
        Some(BuiltIn::MsgValue) => Some(mlir_op_build!(
            state,
            CallValueOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockTimestamp) => Some(mlir_op_build!(
            state,
            TimestampOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockNumber) => Some(mlir_op_build!(
            state,
            BlockNumberOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockCoinbase) => {
            Some(mlir_op_build!(state, CoinbaseOperation.addr(AstType::address(state.mlir_context, false))))
        }
        Some(BuiltIn::BlockChainid) => Some(mlir_op_build!(
            state,
            ChainIdOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBasefee) => Some(mlir_op_build!(
            state,
            BaseFeeOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockGaslimit) => Some(mlir_op_build!(
            state,
            GasLimitOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBlobbasefee) => Some(mlir_op_build!(
            state,
            BlobBaseFeeOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockDifficulty) => Some(mlir_op_build!(
            state,
            DifficultyOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockPrevrandao) => Some(mlir_op_build!(
            state,
            PrevRandaoOperation.val(AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSig) => Some(mlir_op_build!(
            state,
            SigOperation.val(AstType::fixed_bytes(state.mlir_context, 4))
        )),
        Some(BuiltIn::MsgData) => Some(mlir_op_build!(
            state,
            GetCallDataOperation.addr(AstType::string(state.mlir_context, solx_utils::DataLocation::CallData))
        )),
        _ => None,
    };
    if let Some(operation) = environment_op {
        let value: MlirValue<'context, 'block> = block
            .append_operation(operation)
            .result(0)
            .expect("an environment global produces one result")
            .into();
        return BlockAnd {
            block,
            value: value.into(),
        };
    }
    let emit_operand_effect = |block: BlockRef<'context, 'block>| -> BlockRef<'context, 'block> {
        if let Expression::MemberAccessExpression(inner) = node.operand()
            && !MemberAccessOperand(&inner.operand()).is_namespace_or_type()
        {
            inner.operand().emit_for_effect(context, block)
        } else {
            block
        }
    };
    if matches!(node.operand().get_type(), Some(SlangType::Struct(_))) {
        let BlockAnd {
            value: Place {
                address,
                element_type,
            },
            block,
        } = node.emit_place(context, block);
        let value = Pointer::new(address).load(
            AstType::new(element_type),
            context.state,
            &block,
        );
        BlockAnd { block, value }
    } else if let Some(ordinal) = match (
        node.member().resolve_to_definition(),
        MemberAccessOperand(&node.operand()).resolve(),
    ) {
        (
            Some(Definition::EnumMember(member_definition)),
            Some(Definition::Enum(enum_definition)),
        ) => enum_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_definition.node_id()),
        _ => None,
    } {
        let result_type = AstType::resolve_optional(node.get_type(), context.state)
            .expect("slang validated");
        let state = context.state;
        let value = AstValue::uint256(ordinal as i64, state, &block)
            .cast(AstType::new(result_type), state, &block);
        BlockAnd { block, value }
    } else if matches!(&built_in, Some(BuiltIn::FunctionSelector)) {
        let static_selector = match MemberAccessOperand(&node.operand()).resolve() {
            Some(Definition::Function(function)) => function.compute_selector(),
            Some(Definition::StateVariable(state_variable)) => state_variable.compute_selector(),
            _ => None,
        };
        if let Some(selector) = static_selector {
            let block = emit_operand_effect(block);
            let value =
                AstValue::selector_constant(&BigInt::from(selector), 4, context.state, &block);
            return BlockAnd { block, value };
        }
        let BlockAnd {
            value: operand_value,
            block,
        } = node.operand().emit(context, block);
        let value = operand_value.ext_func_selector(context.state, &block);
        BlockAnd { block, value }
    } else {
        let selector_constant = match &built_in {
            Some(BuiltIn::ErrorSelector) => {
                let Some(Definition::Error(error)) = MemberAccessOperand(&node.operand()).resolve()
                else {
                    unreachable!("slang resolves an error `.selector` base to an error definition");
                };
                Some((BigInt::from(error.compute_selector().expect("slang validated")), 4))
            }
            Some(BuiltIn::EventSelector) => {
                let Some(Definition::Event(event)) = MemberAccessOperand(&node.operand()).resolve()
                else {
                    unreachable!("slang resolves an event `.selector` base to an event definition");
                };
                let topic = event.compute_event_topic().expect("slang validated");
                Some((BigInt::from_bytes_be(Sign::Plus, &topic), 32))
            }
            _ => None,
        };
        if let Some((selector, byte_width)) = selector_constant {
            let block = emit_operand_effect(block);
            let value =
                AstValue::selector_constant(&selector, byte_width, context.state, &block);
            return BlockAnd { block, value };
        }
        match &built_in {
            Some(BuiltIn::FunctionAddress) => {
                let BlockAnd {
                    value: operand_value,
                    block,
                } = node.operand().emit(context, block);
                let value = operand_value.ext_func_address(context.state, &block);
                BlockAnd { block, value }
            }
            _ => {
                let member_definition = node.member().resolve_to_definition();
                if let Some(Definition::Library(library)) = &member_definition {
                    let name = solx_utils::ContractName::new(
                        library.get_file_id().to_owned(),
                        Some(library.name().name()),
                    );
                    let value =
                        AstValue::library_address(&name, context.state, &block);
                    return BlockAnd { block, value };
                }
                if let Some(Definition::StateVariable(state_variable)) = &member_definition {
                    let state = context.state;
                    let Some((parameter_types, return_types)) =
                        state_variable.getter_signature(state)
                    else {
                        unreachable!(
                            "a function pointer to a public accessor with no returnable members is invalid"
                        );
                    };
                    let selector = state_variable.compute_selector().expect("slang validated");
                    let BlockAnd {
                        value: receiver,
                        block,
                    } = node.operand().emit(context, block);
                    let value = AstValue::external_callee(
                        receiver,
                        selector,
                        &parameter_types,
                        &return_types,
                        context.state,
                        &block,
                    );
                    return BlockAnd { block, value };
                }
                let Some(Definition::Function(function_definition)) = member_definition else {
                    unreachable!("unsupported member access: {}", node.member().name());
                };
                if let Some(selector) = function_definition.compute_selector() {
                    let (parameter_types, return_types) = AstType::resolve_signature(
                        &function_definition,
                        LocationPolicy::ForceMemory,
                        context.state,
                    );
                    let BlockAnd {
                        value: receiver,
                        block,
                    } = node.operand().emit(context, block);
                    let value = AstValue::external_callee(
                        receiver,
                        selector,
                        &parameter_types,
                        &return_types,
                        context.state,
                        &block,
                    );
                    BlockAnd { block, value }
                } else {
                    let value = context
                        .state
                        .resolve_function(function_definition.node_id())
                        .pointer_constant(context.state, &block);
                    BlockAnd { block, value }
                }
            }
        }
    }
});
