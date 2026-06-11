//!
//! `type(T).min`/`max`/`interfaceId`/`code`/`name` lowering.
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
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits `type(E).min` / `type(E).max` for an enum — the lowest (`0`) or
    /// highest (`member_count - 1`) member ordinal, materialised as an integer
    /// constant and bridged to the enum type via `sol.enum_cast`.
    pub fn emit_type_enum_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
        let result_type = self
            .expression_emitter
            .resolve_slang_type(access.get_type())
            .expect("slang types type(E).min/max as the enum");
        let member_count = enum_definition.members().iter().count();
        let ordinal = match builtin {
            BuiltIn::TypeEnumMin => 0,
            BuiltIn::TypeEnumMax => member_count.saturating_sub(1) as i64,
            _ => unreachable!("dispatched on TypeEnumMin / TypeEnumMax"),
        };
        let builder = &self.expression_emitter.state.builder;
        let int_value = builder.emit_sol_constant(ordinal, builder.types.ui256, &block);
        let enum_value = builder.emit_sol_enum_cast(int_value, result_type, &block);
        Ok((enum_value, block))
    }

    /// Emits `type(T).min` / `type(T).max` for an integer type — a compile-time
    /// integer constant of `T`.
    pub fn emit_type_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builtin = access
            .member()
            .resolve_to_built_in()
            .expect("type(T).min/max dispatches on its built-in member");
        let result_type = self
            .expression_emitter
            .resolve_slang_type(access.get_type())
            .expect("slang types type(T).min/max as the integer type");
        let integer_type =
            IntegerType::try_from(result_type).expect("type(T).min/max is an integer type");
        let bits = solx_mlir::TypeFactory::integer_bit_width(result_type) as usize;
        let value = match (builtin, integer_type.is_signed()) {
            (BuiltIn::TypeMin, false) => BigInt::ZERO,
            (BuiltIn::TypeMin, true) => -(BigInt::from(1) << (bits - 1)),
            (BuiltIn::TypeMax, false) => (BigInt::from(1) << bits) - 1,
            (BuiltIn::TypeMax, true) => (BigInt::from(1) << (bits - 1)) - 1,
            _ => unreachable!("dispatched on TypeMin / TypeMax"),
        };
        let value =
            self.expression_emitter
                .state
                .builder
                .emit_constant(&value, result_type, &block);
        Ok((value, block))
    }

    /// Emits `type(I).interfaceId` (EIP-165): a compile-time `bytes4` constant,
    /// the XOR of the selectors of the functions declared *directly* within the
    /// interface `I` (inherited functions are excluded, matching solc, so the
    /// interface's own members are iterated rather than its linearised set).
    pub fn emit_type_interface_id(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
        let builder = &self.expression_emitter.state.builder;
        let integer_type = Type::from(IntegerType::unsigned(builder.context, 32));
        let integer = builder.emit_constant(&BigInt::from(interface_id), integer_type, &block);
        let value = builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(4), &block);
        Ok((value, block))
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
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
        self.expression_emitter
            .state
            .add_dependency(object_name.clone());
        let result_type = self
            .expression_emitter
            .resolve_slang_type(access.get_type())
            .unwrap_or_else(|| {
                self.expression_emitter
                    .state
                    .builder
                    .types
                    .string(DataLocation::Memory)
            });
        let builder = &self.expression_emitter.state.builder;
        let value = block
            .append_operation(
                ObjectCodeOperation::builder(builder.context, builder.unknown_location)
                    .obj_name(StringAttribute::new(builder.context, &object_name))
                    .out(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.object_code always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits `type(C).name` — the contract / interface name as a `string memory`
    /// constant.
    pub fn emit_type_name(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_string_lit(&type_name, &block);
        Ok((value, block))
    }
}
