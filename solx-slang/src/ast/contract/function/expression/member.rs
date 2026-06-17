//!
//! Member access expression emission: `base.member`. Routes a namespace-
//! qualified state-variable / constant read, a struct field read, and a
//! built-in member access; the struct-field address routine is shared with the
//! lvalue write path.
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
use slang_solidity_v2::ast::PositionalArguments;
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
use solx_mlir::ods::sol::ExtFuncAddrOperation;
use solx_mlir::ods::sol::ExtFuncSelectorOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::GetCallDataOperation;
use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::ObjectCodeOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::PrevRandaoOperation;
use solx_mlir::ods::sol::SigOperation;
use solx_mlir::ods::sol::StringLitOperation;
use solx_mlir::ods::sol::TimestampOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::EmitAddress;
use crate::ast::LocationPolicy;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block, 'scope> EmitAddress<'context, 'block, 'state, 'scope>
    for MemberAccessExpression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;

    /// Emits the address `s.field` denotes together with the field's element MLIR
    /// type (`sol.gep` to the field offset), without the trailing `sol.load`.
    /// Only valid for a struct base.
    fn emit_address(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        let base = self.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            unreachable!("a struct-field address is only emitted for a struct base");
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        // Resolve the accessed field to its `StructMember` definition and locate
        // it by node-id identity — slang exposes struct fields as an ordered list
        // with no direct field-index lookup, but the binder resolves the access,
        // so no name-string comparison is needed.
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
    // A namespace-qualified state-variable / constant read — `C.x`, `L.CONST`,
    // `M.a` — reads the named member exactly like the bare identifier would,
    // disambiguating from a shadowing local. The operand must be a namespace name
    // (a contract / library / import alias); `this.x` keeps the external-getter
    // path since its operand is the `this` keyword, not an identifier.
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
        match node.member().resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let (value, block) = context.emit_state_variable_read(&state_variable, block);
                return BlockAnd {
                    block,
                    value: value.into(),
                };
            }
            Some(Definition::Constant(constant)) => {
                let initializer = constant
                    .value()
                    .expect("slang validated");
                return initializer.emit(context, block);
            }
            _ => {}
        }
    }
    // `type(T).min/max/interfaceId/name/creationCode/runtimeCode`: a
    // compile-time property of the named type, dispatched on slang's typed
    // built-in classification of the member.
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
            let value = AstValue::constant(
                ordinal,
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            )
            .cast(AstType::new(result_type), builder, &block);
            return BlockAnd { block, value };
        }
        Some(BuiltIn::TypeInterfaceId) => {
            // `type(I).interfaceId` (EIP-165): the XOR of the selectors of the
            // functions declared *directly* in interface `I` (inherited ones are
            // excluded, matching solc), a compile-time `bytes4`. `sol.fixedbytes<4>`
            // rejects a bare integer attribute, so emit a `uint32` constant and
            // bridge to `bytes4` (the `f.selector` pattern).
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
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                StringLitOperation
                    .value(StringAttribute::new(builder.context, &type_name))
                    .addr(AstType::string(builder.context, DataLocation::Memory))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        Some(builtin @ (BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode)) => {
            // `type(C).creationCode/runtimeCode` — the contract's deploy / deployed
            // bytecode (`bytes memory`) via `sol.object_code`, referencing the object
            // by name (`C` / `C_deployed`). The reference is a linker dependency so
            // the assembler pulls the object in; the deployed object is distinct, so
            // `runtimeCode` must depend on `C_deployed` (depending on `C` alone leaves
            // its `__datasize__`/`__dataoffset__` symbols unresolved).
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
            let result_type =
                AstType::resolve_optional(node.get_type(), &context.state.builder)
                    .unwrap_or_else(|| {
                        AstType::string(context.state.builder.context, DataLocation::Memory)
                            .into_mlir()
                    });
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
            let builder = &context.state.builder;
            let value: MlirValue<'context, 'block> = mlir_op!(
                builder,
                &block,
                LengthOperation
                    .inp(operand.into_mlir())
                    .len(AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD))
            );
            return BlockAnd {
                block,
                value: value.into(),
            };
        }
        // A function-like built-in member named WITHOUT a call —
        // `addr.transfer`/`send`/`call`/`delegatecall`/`staticcall`, `data.pop`/
        // `push`, e.g. a discarded `data.pop;` — is a reference, not the action
        // (which the call dispatch handles). solc only binds the function;
        // evaluate the operand for its side effects and yield a placeholder.
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
            let value = AstValue::constant(
                0,
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            );
            return BlockAnd { block, value };
        }
        // `abi.encode;` / `T.wrap;` named without a call are no-ops: the operand
        // is the `abi` namespace keyword or the type, not a value (binding it
        // would fail), so nothing is evaluated. Yield a placeholder.
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
            let value = AstValue::constant(
                0,
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &block,
            );
            return BlockAnd { block, value };
        }
        _ => {}
    }
    // EVM environment globals (`block`/`tx`/`msg`): nullary `sol.*` intrinsics.
    // Built eagerly (each is a distinct op type), then appended once.
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
        } = node.emit_address(context, block);
        let value = Pointer::new(address).load(
            AstType::new(element_type),
            &context.state.builder,
            &block,
        );
        BlockAnd { block, value }
    } else if let Some(ordinal) = context.enum_variant_ordinal(node, None) {
        // `E.Variant` (or qualified `C.E.Variant`): the variant's ordinal as an
        // integer constant, bridged to the enum type via `sol.enum_cast`.
        let result_type = AstType::resolve_optional(node.get_type(), &context.state.builder)
            .expect("slang validated");
        let builder = &context.state.builder;
        let value = AstValue::constant(
            ordinal as i64,
            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            &block,
        )
        .cast(AstType::new(result_type), builder, &block);
        BlockAnd { block, value }
    } else {
        // A value-position member built-in over a non-struct base: `.selector` /
        // `.address`, or an external function pointer.
        match node.member().resolve_to_built_in() {
            // `f.selector` — the 4-byte selector (`bytes4`). A statically named
            // function (`this.f`, `i.foo`) or public-getter member folds to a
            // compile-time constant via `compute_selector()`; an external
            // function-pointer VALUE pulls its selector at runtime via
            // `sol.ext_func_selector`.
            Some(BuiltIn::FunctionSelector) => {
                let static_selector = match context.resolve_member_access_operand(&node.operand()) {
                    Some(Definition::Function(function)) => function.compute_selector(),
                    Some(Definition::StateVariable(state_variable)) => {
                        state_variable.compute_selector()
                    }
                    _ => None,
                };
                if let Some(selector) = static_selector {
                    let block = context.eval_selector_receiver_side_effects(node, block);
                    let value = AstValue::selector_constant(
                        &BigInt::from(selector),
                        4,
                        &context.state.builder,
                        &block,
                    );
                    return BlockAnd { block, value };
                }
                let BlockAnd {
                    value: operand_value,
                    block,
                } = node.operand().emit(context, block);
                let value: MlirValue<'context, 'block> = mlir_op!(
                    &context.state.builder,
                    &block,
                    ExtFuncSelectorOperation
                        .func(operand_value)
                        .result(AstType::fixed_bytes(context.state.builder.context, 4))
                );
                BlockAnd {
                    block,
                    value: value.into(),
                }
            }
            // `f.address` — the address component of an external function-pointer
            // VALUE, pulled out of its `!sol.ext_func_ref` via `sol.ext_func_addr`.
            Some(BuiltIn::FunctionAddress) => {
                let BlockAnd {
                    value: operand_value,
                    block,
                } = node.operand().emit(context, block);
                let value: MlirValue<'context, 'block> = mlir_op!(
                    &context.state.builder,
                    &block,
                    ExtFuncAddrOperation
                        .func(operand_value)
                        .result(AstType::address(context.state.builder.context, false))
                );
                BlockAnd {
                    block,
                    value: value.into(),
                }
            }
            // `MyError.selector` — the error's 4-byte selector as a compile-time
            // constant.
            Some(BuiltIn::ErrorSelector) => {
                let Some(Definition::Error(error)) =
                    context.resolve_member_access_operand(&node.operand())
                else {
                    unreachable!("slang resolves an error `.selector` base to an error definition");
                };
                let selector = error.compute_selector().expect("slang validated");
                let block = context.eval_selector_receiver_side_effects(node, block);
                let value = AstValue::selector_constant(
                    &BigInt::from(selector),
                    4,
                    &context.state.builder,
                    &block,
                );
                BlockAnd { block, value }
            }
            // `MyEvent.selector` — the event's 32-byte topic hash (`bytes32`), the
            // keccak256 of its canonical signature, as a compile-time constant.
            Some(BuiltIn::EventSelector) => {
                let Some(Definition::Event(event)) =
                    context.resolve_member_access_operand(&node.operand())
                else {
                    unreachable!("slang resolves an event `.selector` base to an event definition");
                };
                let signature = event.compute_canonical_signature().expect("slang validated");
                let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
                let topic = BigInt::from_bytes_be(Sign::Plus, hash.as_bytes());
                let block = context.eval_selector_receiver_side_effects(node, block);
                let value =
                    AstValue::selector_constant(&topic, 32, &context.state.builder, &block);
                BlockAnd { block, value }
            }
            // A member resolving to a function used as a value (not called) is a
            // function pointer: an externally-visible function with a selector
            // (`this.f`, `instance.f`) is an external pointer, while a
            // namespace-qualified internal function with none (`C.f`, `(L.f)`) is
            // an internal pointer (`sol.func_constant`), like a bare `f`.
            _ => {
                let Some(Definition::Function(function_definition)) =
                    node.member().resolve_to_definition()
                else {
                    unimplemented!("unsupported member access: {}", node.member().name());
                };
                if let Some(selector) = function_definition.compute_selector() {
                    // An external function pointer's ABI representation (address +
                    // selector) types its reference parameters as `Memory`, not their
                    // declared `calldata`/`storage` location — calldata cannot cross
                    // the call boundary, and solc emits the pointer at this memory
                    // signature.
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
                    // The literal target lowers (no virtual redirect): an explicit
                    // `Base.f` names Base's own implementation, not the most-derived
                    // override a bare `f` would bind.
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

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Classifies a member access as an enum-variant reference (`E.Variant` or
    /// qualified `C.E.Variant`), returning the variant's ordinal when it is one
    /// (and not a call). The ordinal is located by NodeId identity against the
    /// enum's members, never by comparing the member name as text.
    fn enum_variant_ordinal(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
    ) -> Option<usize> {
        if arguments.is_some() {
            return None;
        }
        let Definition::EnumMember(member_definition) = access.member().resolve_to_definition()?
        else {
            return None;
        };
        let Definition::Enum(enum_definition) =
            self.resolve_member_access_operand(&access.operand())?
        else {
            return None;
        };
        enum_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_definition.node_id())
    }

    /// Resolves a member-access operand to its definition: a bare type name
    /// (`E.Variant`, whose operand is the `Identifier` `E`) or a qualified path
    /// whose operand is itself a member access (`C.E.Variant`).
    fn resolve_member_access_operand(&self, operand: &Expression) -> Option<Definition> {
        match operand {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }

    /// Evaluates the receiver of a `<receiver>.member.selector` for its side
    /// effects when `<receiver>` is a runtime value (e.g. the call in
    /// `h().f.selector`). A namespace / type qualifier (`C.f.selector`) has no
    /// runtime value, so nothing is evaluated. The selector itself stays a
    /// compile-time constant; this only reproduces the discarded receiver's
    /// evaluation.
    fn eval_selector_receiver_side_effects(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let Expression::MemberAccessExpression(inner) = access.operand() else {
            return block;
        };
        let receiver = inner.operand();
        if self.is_namespace_or_type_operand(&receiver) {
            return block;
        }
        let BlockAnd {
            value: _discarded,
            block,
        } = receiver.emit(self, block);
        block
    }

    /// Whether `expression` is a namespace or type reference (a contract /
    /// interface / library / import / enum / struct / user-defined-value-type
    /// name) rather than a runtime value — such an operand carries no side
    /// effects, so a `.selector` taken through it evaluates nothing.
    fn is_namespace_or_type_operand(&self, expression: &Expression) -> bool {
        matches!(
            self.resolve_member_access_operand(expression),
            Some(
                Definition::Contract(_)
                    | Definition::Interface(_)
                    | Definition::Library(_)
                    | Definition::Import(_)
                    | Definition::ImportedSymbol(_)
                    | Definition::Enum(_)
                    | Definition::Struct(_)
                    | Definition::UserDefinedValueType(_)
            )
        )
    }
}
