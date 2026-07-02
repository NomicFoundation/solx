//!
//! Member access expression lowering: struct fields, namespace-qualified reads, type metadata,
//! selectors, and function-pointer values.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ObjectCodeOperation;

use crate::ast::analysis::query::member_access_operand::MemberAccessOperand;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_place::EmitPlace;
use crate::ast::place::Place;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for MemberAccessExpression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

    /// Lowers `base.member` in value position: a struct-field load, a namespace-qualified state or
    /// constant read (`C.x`), `type(T)` metadata, an enum-variant ordinal, a compile-time selector,
    /// or an internal function-pointer value; anything else is a built-in intrinsic (`msg.sender`).
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        if let Some(BlockAnd {
            value: place,
            block,
        }) = context.emit_struct_field_place(self, block)
        {
            let value = Pointer::new(place.address)
                .load(AstType::new(place.element_type), context.state, &block)
                .into_mlir();
            return BlockAnd { block, value };
        }
        if let Expression::Identifier(operand) = self.operand()
            && matches!(
                operand.resolve_to_definition(),
                Some(
                    Definition::Contract(_)
                        | Definition::Library(_)
                        | Definition::Import(_)
                        | Definition::ImportedSymbol(_)
                )
            )
            && matches!(
                self.member().resolve_to_definition(),
                Some(Definition::StateVariable(_) | Definition::Constant(_))
            )
        {
            return self.member().emit(context, block);
        }
        if let Some(result) = context.emit_type_member(self, block) {
            return result;
        }
        if let Some(ordinal) = context.enum_ordinal(self) {
            let result_type = context
                .resolve_slang_type(self.get_type())
                .expect("slang types every enumeration value");
            let value = AstValue::constant(
                ordinal,
                AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                context.state,
                &block,
            )
            .enum_cast(AstType::new(result_type), context.state, &block)
            .into_mlir();
            return BlockAnd { block, value };
        }
        if let Some(result) = context.emit_selector(self, block) {
            return result;
        }
        if let Some(Definition::Function(function_definition)) =
            self.member().resolve_to_definition()
        {
            let value = context
                .state
                .function_signatures
                .get(&function_definition.node_id())
                .expect("a namespace-qualified internal function resolves to a registered signature")
                .pointer_constant(context.state, &block)
                .into_mlir();
            return BlockAnd { block, value };
        }
        let (value, block) = CallContext::new(context).emit_member_access(self, block);
        BlockAnd { block, value }
    }
}

impl<'context: 'block, 'block> EmitPlace<'context, 'block> for MemberAccessExpression {
    /// Emits the address yielded by `s.field` together with the field's element MLIR type, without
    /// the trailing `sol.load`. Panics when the base is not a struct, which the assignment lvalue
    /// path guarantees.
    fn emit_place<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Place<'context, 'block>> {
        context
            .emit_struct_field_place(self, block)
            .expect("slang validates a member-access lvalue resolves to a struct field")
    }
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// The enumeration ordinal an enum-valued member access denotes, or `None` when it is not one: an
    /// enum member `E.Variant` yields the variant's declaration index, and `type(E).min` / `type(E).max`
    /// the first and last ordinals.
    fn enum_ordinal(&self, access: &MemberAccessExpression) -> Option<i64> {
        match access.member().resolve_to_built_in() {
            Some(builtin @ (BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax)) => {
                let Expression::TypeExpression(type_expression) = access.operand() else {
                    unreachable!("a type(...) builtin operand is a type expression");
                };
                let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
                else {
                    unreachable!("type(E) on an enumeration names it via an identifier path");
                };
                let Some(Definition::Enum(enum_definition)) =
                    identifier_path.resolve_to_definition()
                else {
                    unreachable!("type(E).min/max resolves to an enum definition");
                };
                let member_count = enum_definition.members().iter().count();
                Some(match builtin {
                    BuiltIn::TypeEnumMin => 0,
                    BuiltIn::TypeEnumMax => (member_count - 1) as i64,
                    _ => unreachable!("dispatched on TypeEnumMin / TypeEnumMax"),
                })
            }
            None => {
                let Some(Definition::EnumMember(member_identifier)) =
                    access.member().resolve_to_definition()
                else {
                    return None;
                };
                let Some(SlangType::Enum(enum_type)) = access.get_type() else {
                    return None;
                };
                let Definition::Enum(enum_definition) = enum_type.definition() else {
                    unreachable!("slang EnumType always references an Enum definition");
                };
                enum_definition
                    .members()
                    .iter()
                    .position(|member| member.node_id() == member_identifier.node_id())
                    .map(|ordinal| ordinal as i64)
            }
            _ => None,
        }
    }

    /// Emits a `type(T)` metadata member — `min`/`max` for an integer type, `interfaceId` for an
    /// interface, `name` for a contract or interface, and `creationCode`/`runtimeCode` for a
    /// contract — returning `None` when the member is not one of these type-level properties.
    fn emit_type_member(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockAnd<'context, 'block, Value<'context, 'block>>> {
        match access.member().resolve_to_built_in() {
            Some(builtin @ (BuiltIn::TypeMin | BuiltIn::TypeMax)) => {
                let result_type = self
                    .resolve_slang_type(access.get_type())
                    .expect("slang types every type(T).min/max");
                let integer_type =
                    IntegerType::try_from(result_type).expect("type(T).min/max names an integer");
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
                    self.state,
                    &block,
                )
                .into_mlir();
                Some(BlockAnd { block, value })
            }
            Some(BuiltIn::TypeInterfaceId) => {
                let Some(Definition::Interface(interface_definition)) =
                    Self::type_expression_definition(access)
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
                let value = AstValue::constant(
                    i64::from(interface_id),
                    AstType::new(Type::from(IntegerType::unsigned(self.state.mlir_context, 32))),
                    self.state,
                    &block,
                )
                .bytes_cast(AstType::fixed_bytes(self.state.mlir_context, 4), self.state, &block)
                .into_mlir();
                Some(BlockAnd { block, value })
            }
            Some(BuiltIn::TypeName) => {
                let type_name = match Self::type_expression_definition(access) {
                    Some(Definition::Contract(contract)) => contract.name().name(),
                    Some(Definition::Interface(interface)) => interface.name().name(),
                    _ => unreachable!("type(C).name resolves to a contract or interface"),
                };
                let value =
                    AstValue::string_literal(&type_name, self.state, &block).into_mlir();
                Some(BlockAnd { block, value })
            }
            Some(builtin @ (BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode)) => {
                let Some(Definition::Contract(contract_definition)) =
                    Self::type_expression_definition(access)
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
                self.state.add_dependency(object_name.clone());
                let result_type = self
                    .resolve_slang_type(access.get_type())
                    .expect("slang types type(C).creationCode/runtimeCode");
                let value: Value<'context, 'block> = mlir_op!(
                    self.state,
                    &block,
                    ObjectCodeOperation
                        .obj_name(StringAttribute::new(self.state.mlir_context, &object_name))
                        .out(result_type)
                );
                Some(BlockAnd { block, value })
            }
            _ => None,
        }
    }

    /// The definition named by a `type(T)` operand's identifier path.
    fn type_expression_definition(access: &MemberAccessExpression) -> Option<Definition> {
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("a type(...) builtin operand is a type expression");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(T) names a user-defined type via an identifier path");
        };
        identifier_path.resolve_to_definition()
    }

    /// Emits a compile-time `.selector` — a function or error four-byte selector, or an event's
    /// thirty-two-byte topic — running the receiver's side effects first, or `None` when the member
    /// is not a static selector.
    fn emit_selector(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockAnd<'context, 'block, Value<'context, 'block>>> {
        let (selector, byte_width) = match access.member().resolve_to_built_in() {
            Some(BuiltIn::FunctionSelector) => {
                let Some(Definition::Function(function)) =
                    MemberAccessOperand(&access.operand()).resolve()
                else {
                    return None;
                };
                (BigInt::from(function.compute_selector()?), 4)
            }
            Some(BuiltIn::ErrorSelector) => {
                let Some(Definition::Error(error)) =
                    MemberAccessOperand(&access.operand()).resolve()
                else {
                    unreachable!("slang resolves an error `.selector` base to an error definition");
                };
                (
                    BigInt::from(error.compute_selector().expect("slang validated")),
                    4,
                )
            }
            Some(BuiltIn::EventSelector) => {
                let Some(Definition::Event(event)) =
                    MemberAccessOperand(&access.operand()).resolve()
                else {
                    unreachable!("slang resolves an event `.selector` base to an event definition");
                };
                let signature = event.compute_canonical_signature().expect("slang validated");
                let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
                (BigInt::from_bytes_be(Sign::Plus, hash.as_bytes()), 32)
            }
            _ => return None,
        };
        let block = if let Expression::MemberAccessExpression(inner) = access.operand()
            && !MemberAccessOperand(&inner.operand()).is_namespace_or_type()
        {
            inner.operand().emit_for_effect(self, block)
        } else {
            block
        };
        let integer_width = (byte_width * 8) as usize;
        let value = AstValue::constant_from_bigint(
            &selector,
            AstType::unsigned(self.state.mlir_context, integer_width),
            self.state,
            &block,
        )
        .bytes_cast(
            AstType::fixed_bytes(self.state.mlir_context, byte_width),
            self.state,
            &block,
        )
        .into_mlir();
        Some(BlockAnd { block, value })
    }

    /// Emits the address yielded by `s.field` when the base is a struct, returning `None` otherwise
    /// so the caller can fall back to built-in member-access lowering.
    fn emit_struct_field_place(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockAnd<'context, 'block, Place<'context, 'block>>> {
        let base = access.operand();
        let SlangType::Struct(struct_type) = base.get_type()? else {
            return None;
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        let member_name = access.member().name();
        let field_index = struct_definition
            .members()
            .iter()
            .position(|member| member.name().name() == member_name)
            .expect("slang validates the accessed member exists");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(self, block);

        let index_value = AstValue::constant(
            field_index as i64,
            AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_X64),
            self.state,
            &block,
        );
        let element_type = unsafe {
            Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                base_value.r#type().to_raw(),
                field_index as u64,
            ))
        };
        let address = Pointer::new(base_value)
            .gep(index_value, AstType::new(element_type), self.state, &block)
            .into_mlir();
        Some(BlockAnd {
            block,
            value: Place {
                address,
                element_type,
            },
        })
    }
}
