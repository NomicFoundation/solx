//!
//! Member access expression emission: `base.member` — namespace-qualified reads, struct fields, and built-ins.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
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
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::EmitPlace;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::getter::GetterSignature;
use crate::ast::analysis::query::MemberAccessOperand;

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for MemberAccessExpression {
    /// Emits the address `s.field` denotes with the field's element type (`sol.gep`), without the load. Struct base only.
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

        // Locate the accessed field by node-id identity — slang exposes fields as an ordered list
        // with no direct index lookup, and the binder resolves the access (no name comparison).
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
        let builder = &context.state.builder;

        let index_value = AstValue::constant(
            field_index as i64,
            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
            builder,
            &block,
        );
        let element_type = base_value.r#type().element_type(field_index);
        let address = base_value
            .into_pointer()
            .gep(index_value, element_type, builder, &block)
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
    // A namespace-qualified read (`C.x`, `L.CONST`, `M.a`) reads the named member like the bare
    // identifier would. `this.x` keeps the external-getter path (its operand is `this`, not an identifier).
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
        // A state-variable / `constant` member reads as the bare identifier would (keyed by node id).
        // A `Function` member is NOT delegated — it would pick up the virtual redirect `Base.f` must skip.
        match node.member().resolve_to_definition() {
            Some(Definition::StateVariable(_) | Definition::Constant(_)) => {
                return node.member().emit(context, block);
            }
            _ => {}
        }
    }
    // `type(T).min/max/interfaceId/name/creationCode/runtimeCode`: a compile-time property of the named type.
    match node.member().resolve_to_built_in() {
        Some(builtin @ (BuiltIn::TypeMin | BuiltIn::TypeMax)) => {
            // `type(T).min/max` for an integer type is a compile-time integer
            // constant of `T`.
            let result_type =
                AstType::resolve_optional(node.get_type(), &context.state.builder)
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
                &context.state.builder,
                &block,
            );
            return BlockAnd { block, value };
        }
        Some(builtin @ (BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax)) => {
            // `type(E).min/max` for an enum is the lowest (`0`) or highest
            // (`member_count - 1`) member ordinal, bridged to the enum type via
            // `sol.enum_cast`.
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
                AstType::resolve_optional(node.get_type(), &context.state.builder)
                    .expect("slang validated");
            let member_count = enum_definition.members().iter().count();
            let ordinal = match builtin {
                BuiltIn::TypeEnumMin => 0,
                BuiltIn::TypeEnumMax => member_count.saturating_sub(1) as i64,
                _ => unreachable!("dispatched on TypeEnumMin / TypeEnumMax"),
            };
            let builder = &context.state.builder;
            let value = AstValue::uint256(ordinal, builder, &block)
                .cast(AstType::new(result_type), builder, &block);
            return BlockAnd { block, value };
        }
        Some(BuiltIn::TypeInterfaceId) => {
            // `type(I).interfaceId` (EIP-165): the XOR of the selectors of `I`'s *directly*-declared
            // functions, a compile-time `bytes4` (emitted as a `uint32` constant bridged to `bytes4`).
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
            let builder = &context.state.builder;
            let integer_type = Type::from(IntegerType::unsigned(builder.context, 32));
            let value = AstValue::constant_from_bigint(
                &BigInt::from(interface_id),
                AstType::new(integer_type),
                builder,
                &block,
            )
            .cast(AstType::fixed_bytes(builder.context, 4), builder, &block);
            return BlockAnd { block, value };
        }
        Some(BuiltIn::TypeName) => {
            // `type(C).name` — the contract / interface name as a `string memory`
            // constant.
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
            let value = AstValue::string_literal(&type_name, &context.state.builder, &block);
            return BlockAnd { block, value };
        }
        Some(builtin @ (BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode)) => {
            // `type(C).creationCode/runtimeCode` — the deploy / deployed bytecode (`bytes memory`) via
            // `sol.object_code`. The reference is a linker dependency; `runtimeCode` must depend on
            // `C_deployed` (depending on `C` alone leaves its data symbols unresolved).
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
            let object_name = match builtin {
                BuiltIn::TypeRuntimeCode => {
                    format!("{contract_name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX)
                }
                _ => contract_name,
            };
            context.state.add_dependency(object_name.clone());
            let result_type = AstType::resolve_optional(node.get_type(), &context.state.builder)
                .expect("slang validated");
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                ObjectCodeOperation
                    .obj_name(StringAttribute::new(builder.context, &object_name))
                    .out(result_type)
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        // `address.balance`/`codehash`/`code`, `arr.length`: a unary intrinsic
        // over the receiver value.
        Some(BuiltIn::AddressBalance) => {
            let BlockAnd {
                value: address,
                block,
            } = node.operand().emit(context, block);
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                BalanceOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
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
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                CodeHashOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
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
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                CodeOperation
                    .cont_addr(address.into_mlir())
                    .out(AstType::string(builder.context, DataLocation::Memory))
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
                value: operand.length(&context.state.builder, &block),
                block,
            };
        }
        // A function-like built-in member named WITHOUT a call (`addr.transfer`, `data.pop`, …) is a
        // reference, not the action: evaluate the operand for side effects and yield a placeholder.
        Some(
            BuiltIn::AddressTransfer
            | BuiltIn::AddressSend
            | BuiltIn::AddressCall
            | BuiltIn::AddressDelegatecall
            | BuiltIn::AddressStaticcall
            | BuiltIn::ArrayPop
            | BuiltIn::ArrayPush,
        ) => {
            let BlockAnd {
                value: _operand,
                block,
            } = node.operand().emit(context, block);
            let builder = &context.state.builder;
            let value = AstValue::uint256(0, builder, &block);
            return BlockAnd { block, value };
        }
        // `abi.encode;` / `T.wrap;` named without a call are no-ops (the operand is a namespace / type,
        // not a value), so nothing is evaluated; yield a placeholder.
        Some(
            BuiltIn::AbiEncode
            | BuiltIn::AbiEncodePacked
            | BuiltIn::AbiEncodeWithSelector
            | BuiltIn::AbiEncodeWithSignature
            | BuiltIn::AbiEncodeCall
            | BuiltIn::AbiDecode
            | BuiltIn::Wrap
            | BuiltIn::Unwrap,
        ) => {
            let builder = &context.state.builder;
            let value = AstValue::uint256(0, builder, &block);
            return BlockAnd { block, value };
        }
        _ => {}
    }
    // EVM environment globals (`block`/`tx`/`msg`): nullary `sol.*` intrinsics, built then appended once.
    let builder = &context.state.builder;
    let environment_op = match node.member().resolve_to_built_in() {
        Some(BuiltIn::TxOrigin) => {
            Some(mlir_op_build!(builder, OriginOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::TxGasPrice) => Some(mlir_op_build!(
            builder,
            GasPriceOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSender) => {
            Some(mlir_op_build!(builder, CallerOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::MsgValue) => Some(mlir_op_build!(
            builder,
            CallValueOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockTimestamp) => Some(mlir_op_build!(
            builder,
            TimestampOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockNumber) => Some(mlir_op_build!(
            builder,
            BlockNumberOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockCoinbase) => {
            Some(mlir_op_build!(builder, CoinbaseOperation.addr(AstType::address(builder.context, false))))
        }
        Some(BuiltIn::BlockChainid) => Some(mlir_op_build!(
            builder,
            ChainIdOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBasefee) => Some(mlir_op_build!(
            builder,
            BaseFeeOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockGaslimit) => Some(mlir_op_build!(
            builder,
            GasLimitOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockBlobbasefee) => Some(mlir_op_build!(
            builder,
            BlobBaseFeeOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockDifficulty) => Some(mlir_op_build!(
            builder,
            DifficultyOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::BlockPrevrandao) => Some(mlir_op_build!(
            builder,
            PrevRandaoOperation.val(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
        )),
        Some(BuiltIn::MsgSig) => Some(mlir_op_build!(
            builder,
            SigOperation.val(AstType::fixed_bytes(builder.context, 4))
        )),
        Some(BuiltIn::MsgData) => Some(mlir_op_build!(
            builder,
            GetCallDataOperation.addr(AstType::string(builder.context, DataLocation::CallData))
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
    // A struct-typed base is a field read (`s.field`); anything else
    // (e.g. `msg.sender`, `addr.balance`) is a built-in member access.
    if matches!(node.operand().get_type(), Some(SlangType::Struct(_))) {
        // Address the field (`sol.gep`) and `sol.load` it.
        let BlockAnd {
            value: Place {
                address,
                element_type,
            },
            block,
        } = node.emit_place(context, block);
        let value = Pointer::new(address).load(
            AstType::new(element_type),
            &context.state.builder,
            &block,
        );
        BlockAnd { block, value }
    } else if let Some(ordinal) = match (
        node.member().resolve_to_definition(),
        MemberAccessOperand(&node.operand()).resolve(),
    ) {
        // `E.Variant` (or qualified `C.E.Variant`) not in call position: the
        // variant's ordinal, located by NodeId identity against the enum's members
        // (never by comparing the member name as text).
        (
            Some(Definition::EnumMember(member_definition)),
            Some(Definition::Enum(enum_definition)),
        ) => enum_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_definition.node_id()),
        _ => None,
    } {
        // The variant's ordinal as an integer constant, bridged to the enum type
        // via `sol.enum_cast`.
        let result_type = AstType::resolve_optional(node.get_type(), &context.state.builder)
            .expect("slang validated");
        let builder = &context.state.builder;
        let value = AstValue::uint256(ordinal as i64, builder, &block)
            .cast(AstType::new(result_type), builder, &block);
        BlockAnd { block, value }
    } else {
        // A value-position member built-in over a non-struct base: `.selector` / `.address`, or an
        // external function pointer. Static `.selector` constants (function/error 4-byte, event 32-byte
        // topic) are reached the same way, preceded by the receiver's side effects.
        let selector_constant = match node.member().resolve_to_built_in() {
            Some(BuiltIn::FunctionSelector) => match MemberAccessOperand(&node.operand()).resolve() {
                Some(Definition::Function(function)) => {
                    crate::ast::contract::function::signature::library_aware_selector(&function)
                        .map(|selector| (BigInt::from(selector), 4))
                }
                Some(Definition::StateVariable(state_variable)) => {
                    state_variable.compute_selector().map(|selector| (BigInt::from(selector), 4))
                }
                _ => None,
            },
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
                let signature = event.compute_canonical_signature().expect("slang validated");
                let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
                Some((BigInt::from_bytes_be(Sign::Plus, hash.as_bytes()), 32))
            }
            _ => None,
        };
        if let Some((selector, byte_width)) = selector_constant {
            // The runtime receiver of `<receiver>.member.selector` still runs for its side effects;
            // a namespace / type qualifier (`C.f.selector`) is no value and runs nothing.
            let block = if let Expression::MemberAccessExpression(inner) = node.operand()
                && !MemberAccessOperand(&inner.operand()).is_namespace_or_type()
            {
                inner.operand().emit_for_effect(context, block)
            } else {
                block
            };
            let value =
                AstValue::selector_constant(&selector, byte_width, &context.state.builder, &block);
            return BlockAnd { block, value };
        }
        match node.member().resolve_to_built_in() {
            // `f.selector` on an external function-pointer VALUE pulls its selector
            // at runtime via `sol.ext_func_selector`.
            Some(BuiltIn::FunctionSelector) => {
                let BlockAnd {
                    value: operand_value,
                    block,
                } = node.operand().emit(context, block);
                let value = operand_value.ext_func_selector(&context.state.builder, &block);
                BlockAnd { block, value }
            }
            // `f.address` — the address component of an external function-pointer
            // VALUE, pulled out of its `!sol.ext_func_ref` via `sol.ext_func_addr`.
            Some(BuiltIn::FunctionAddress) => {
                let BlockAnd {
                    value: operand_value,
                    block,
                } = node.operand().emit(context, block);
                let value = operand_value.ext_func_address(&context.state.builder, &block);
                BlockAnd { block, value }
            }
            // A member resolving to a function used as a value (not called) is a
            // function pointer: an externally-visible function with a selector
            // (`this.f`, `instance.f`) is an external pointer, while a
            // namespace-qualified internal function with none (`C.f`, `(L.f)`) is
            // an internal pointer (`sol.func_constant`), like a bare `f`.
            _ => {
                let member_definition = node.member().resolve_to_definition();
                // A member resolving to a library used as a VALUE (`address(M.L)` through an
                // aliased import `import "x" as M; M.L`) is its linked deploy address — the same
                // `sol.lib_addr` a bare `L` emits (`identifier.rs`). solc's MLIR frontend does not
                // resolve the alias here, so this is a solx-only path emitting the canonical op.
                if let Some(Definition::Library(library)) = &member_definition {
                    let name = solx_utils::ContractName::new(
                        library.get_file_id().to_owned(),
                        Some(library.name().name()),
                    );
                    let value =
                        AstValue::library_address(&name, &context.state.builder, &block);
                    return BlockAnd { block, value };
                }
                // A public state variable used as a value is its synthesised external getter
                // taken as a function pointer (`fp = inst.x`): an `sol.ext_func_ref` carrying the
                // getter's selector and ABI signature, mirroring the external-function arm below.
                if let Some(Definition::StateVariable(state_variable)) = &member_definition {
                    let builder = &context.state.builder;
                    let Some((parameter_types, return_types)) =
                        state_variable.getter_signature(builder)
                    else {
                        unimplemented!(
                            "function pointer to a nested or reference-typed getter is not yet supported"
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
                        &context.state.builder,
                        &block,
                    );
                    return BlockAnd { block, value };
                }
                let Some(Definition::Function(function_definition)) = member_definition else {
                    unreachable!("unsupported member access: {}", node.member().name());
                };
                if let Some(selector) = function_definition.compute_selector() {
                    // An external function pointer's ABI representation types its reference parameters
                    // as `Memory` (calldata cannot cross the call boundary), matching solc.
                    let (parameter_types, return_types) = AstType::resolve_signature(
                        &function_definition,
                        LocationPolicy::ForceMemory,
                        &context.state.builder,
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
                        &context.state.builder,
                        &block,
                    );
                    BlockAnd { block, value }
                } else {
                    // The literal target lowers with no virtual redirect: `Base.f` names Base's own
                    // implementation, not the most-derived override a bare `f` would bind.
                    let value = context
                        .state
                        .resolve_function(function_definition.node_id())
                        .pointer_constant(&context.state.builder, &block);
                    BlockAnd { block, value }
                }
            }
        }
    }
});
