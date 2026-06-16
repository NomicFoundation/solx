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
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_mlir::ods::sol::ObjectCodeOperation;
use solx_mlir::ods::sol::StringLitOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::EmitAddress;
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
            let value: MlirValue<'context, 'block> = sol_op!(
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
            let value: MlirValue<'context, 'block> = sol_op!(
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
        _ => {}
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
    } else {
        // `msg.sender`, `addr.balance`, `arr.length`: a built-in member access,
        // which in value position always yields a value.
        let (value, block) = context.emit_built_in_member_access(node, None, block);
        BlockAnd {
            block,
            value: value.expect("a bare member access yields a value").into(),
        }
    }
});
