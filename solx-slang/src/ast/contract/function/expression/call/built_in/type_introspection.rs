//!
//! `type(T).min`/`max`/`interfaceId`/`code`/`name` emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_mlir::ods::sol::ObjectCodeOperation;
use solx_mlir::ods::sol::StringLitOperation;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits `type(E).min` / `type(E).max` for an enum — the lowest (`0`) or
    /// highest (`member_count - 1`) member ordinal, materialised as an integer
    /// constant and bridged to the enum type via `sol.enum_cast`.
    pub fn emit_type_enum_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let builtin = access
            .member()
            .resolve_to_built_in()
            .expect("type(E).min/max dispatches on its built-in member");
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(E).min/max operand is a type expression");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(E) names an enum via an identifier path");
        };
        let Some(Definition::Enum(enum_definition)) = identifier_path.resolve_to_definition()
        else {
            unreachable!("type(E).min/max resolves to an enum definition");
        };
        let result_type =
            crate::ast::Type::resolve_optional(access.get_type(), &self.state.builder)
                .expect("slang types type(E).min/max as the enum");
        let member_count = enum_definition.members().iter().count();
        let ordinal = match builtin {
            BuiltIn::TypeEnumMin => 0,
            BuiltIn::TypeEnumMax => member_count.saturating_sub(1) as i64,
            _ => unreachable!("dispatched on TypeEnumMin / TypeEnumMax"),
        };
        let builder = &self.state.builder;
        let int_value = crate::ast::Value::constant(
            ordinal,
            crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            &block,
        );
        let enum_value = int_value
            .cast(crate::ast::Type::new(result_type), builder, &block)
            .into_mlir();
        (enum_value, block)
    }

    /// Emits `type(T).min` / `type(T).max` for an integer type — a compile-time
    /// integer constant of `T`.
    pub fn emit_type_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let builtin = access
            .member()
            .resolve_to_built_in()
            .expect("type(T).min/max dispatches on its built-in member");
        let result_type =
            crate::ast::Type::resolve_optional(access.get_type(), &self.state.builder)
                .expect("slang types type(T).min/max as the integer type");
        let integer_type =
            IntegerType::try_from(result_type).expect("type(T).min/max is an integer type");
        let bits = crate::ast::Type::new(result_type).integer_bit_width() as usize;
        let value = match (builtin, integer_type.is_signed()) {
            (BuiltIn::TypeMin, false) => BigInt::ZERO,
            (BuiltIn::TypeMin, true) => -(BigInt::from(1) << (bits - 1)),
            (BuiltIn::TypeMax, false) => (BigInt::from(1) << bits) - 1,
            (BuiltIn::TypeMax, true) => (BigInt::from(1) << (bits - 1)) - 1,
            _ => unreachable!("dispatched on TypeMin / TypeMax"),
        };
        let value = crate::ast::Value::constant_from_bigint(
            &value,
            crate::ast::Type::new(result_type),
            &self.state.builder,
            &block,
        )
        .into_mlir();
        (value, block)
    }

    /// Emits `type(I).interfaceId` (EIP-165): a compile-time `bytes4` constant,
    /// the XOR of the selectors of the functions declared *directly* within the
    /// interface `I` (inherited functions are excluded, matching solc, so the
    /// interface's own members are iterated rather than its linearised set).
    pub fn emit_type_interface_id(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(I).interfaceId operand is a type expression");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(I) names an interface via an identifier path");
        };
        let Some(Definition::Interface(interface_definition)) =
            identifier_path.resolve_to_definition()
        else {
            unreachable!("type(I).interfaceId resolves to an interface definition");
        };
        let mut interface_id: u32 = 0;
        for member in interface_definition.members().iter() {
            if let ContractMember::FunctionDefinition(function) = member
                && let Some(selector) = function.compute_selector()
            {
                interface_id ^= selector;
            }
        }
        // `!sol.fixedbytes<4>` rejects a bare integer attribute, so emit the
        // identifier as a `uint32` constant and bridge to `bytes4` via
        // `sol.bytes_cast` (the same pattern as `f.selector`).
        let builder = &self.state.builder;
        let integer_type = Type::from(IntegerType::unsigned(builder.context, 32));
        let integer = crate::ast::Value::constant_from_bigint(
            &BigInt::from(interface_id),
            crate::ast::Type::new(integer_type),
            builder,
            &block,
        );
        let value = integer
            .cast(
                crate::ast::Type::fixed_bytes(builder.context, 4),
                builder,
                &block,
            )
            .into_mlir();
        (value, block)
    }

    /// Emits `type(C).creationCode` / `type(C).runtimeCode` as the contract's
    /// deploy / deployed bytecode (`bytes memory`), lowered to `sol.object_code`
    /// referencing the object by name — `C` for creation, `C_deployed` for
    /// runtime. The reference is registered as a linker dependency so the
    /// assembler pulls the object in (as `new C()` does); the deployed object is
    /// a distinct top-level object, so `runtimeCode` must depend on `C_deployed`
    /// — depending on `C` alone leaves its `__datasize__`/`__dataoffset__`
    /// symbols unresolved.
    pub fn emit_type_code(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let builtin = access
            .member()
            .resolve_to_built_in()
            .expect("type(C).creationCode/runtimeCode dispatches on its built-in member");
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(C).creationCode/runtimeCode operand is a type expression");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
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
                format!(
                    "{contract_name}{}",
                    solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX
                )
            }
            _ => contract_name,
        };
        self.state.add_dependency(object_name.clone());
        let result_type =
            crate::ast::Type::resolve_optional(access.get_type(), &self.state.builder)
                .unwrap_or_else(|| {
                    crate::ast::Type::string(self.state.builder.context, DataLocation::Memory)
                        .into_mlir()
                });
        let builder = &self.state.builder;
        let value = sol_op!(
            builder,
            block,
            ObjectCodeOperation
                .obj_name(StringAttribute::new(builder.context, &object_name))
                .out(result_type)
        );
        (value, block)
    }

    /// Emits `type(C).name` — the contract / interface name as a `string memory`
    /// constant.
    pub fn emit_type_name(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(C).name operand is a type expression");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(C) names a contract via an identifier path");
        };
        let type_name = match identifier_path.resolve_to_definition() {
            Some(Definition::Contract(contract)) => contract.name().name(),
            Some(Definition::Interface(interface)) => interface.name().name(),
            _ => unreachable!("type(C).name resolves to a contract or interface"),
        };
        let value = sol_op!(
            &self.state.builder,
            &block,
            StringLitOperation
                .value(StringAttribute::new(self.state.builder.context, &type_name))
                .addr(
                    crate::ast::Type::string(
                        self.state.builder.context,
                        solx_utils::DataLocation::Memory
                    )
                    .into_mlir()
                )
        );
        (value, block)
    }
}
