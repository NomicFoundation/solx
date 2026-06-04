//!
//! `type(T)` introspection built-ins: `type(T).min` / `.max` for integers,
//! `type(E).min` / `.max` for enums, and `type(I).interfaceId`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Tries to lower a `type(T).member` introspection access to its
    /// compile-time constant. Returns `Ok(None)` when the member is not a
    /// type-introspection built-in handled here (`name`, `creationCode`, and
    /// `runtimeCode` defer to later domains), so the caller falls through.
    pub fn try_emit_type_introspection(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::TypeMin) => self.emit_type_min_max(access, false, block).map(Some),
            Some(BuiltIn::TypeMax) => self.emit_type_min_max(access, true, block).map(Some),
            Some(BuiltIn::TypeEnumMin) => {
                self.emit_type_enum_min_max(access, false, block).map(Some)
            }
            Some(BuiltIn::TypeEnumMax) => {
                self.emit_type_enum_min_max(access, true, block).map(Some)
            }
            Some(BuiltIn::TypeInterfaceId) => self.emit_type_interface_id(access, block).map(Some),
            _ => Ok(None),
        }
    }

    /// `type(T).min` / `type(T).max` for an integer type are compile-time
    /// constants derived from the type's bit width and signedness.
    fn emit_type_min_max(
        &self,
        access: &MemberAccessExpression,
        is_max: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_slang_type = access
            .get_type()
            .expect("the binder types every type(T).min/max");
        let result_type =
            TypeConversion::resolve_slang_type(&result_slang_type, None, &self.state.builder);
        let integer_type =
            IntegerType::try_from(result_type).expect("type(T).min/max names an integer type");
        let bits = solx_mlir::TypeFactory::integer_bit_width(result_type) as usize;
        let value = match (is_max, integer_type.is_signed()) {
            (false, false) => BigInt::ZERO,
            (false, true) => -(BigInt::from(1) << (bits - 1)),
            (true, false) => (BigInt::from(1) << bits) - 1,
            (true, true) => (BigInt::from(1) << (bits - 1)) - 1,
        };
        let constant = self
            .state
            .builder
            .emit_constant(&value, result_type, &block);
        Ok((constant, block))
    }

    /// `type(E).min` / `type(E).max` for an enum are its first / last
    /// enumerator (ordinals `0` and `member_count - 1`).
    fn emit_type_enum_min_max(
        &self,
        access: &MemberAccessExpression,
        is_max: bool,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(E).min/max has a type-expression operand");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(E).min/max names an enum by identifier path");
        };
        let Some(Definition::Enum(enum_definition)) = identifier_path.resolve_to_definition()
        else {
            unreachable!("type(E).min/max resolves to an enum definition");
        };
        let result_slang_type = access
            .get_type()
            .expect("the binder types every type(E).min/max");
        let result_type =
            TypeConversion::resolve_slang_type(&result_slang_type, None, &self.state.builder);
        let member_count = enum_definition.members().iter().count();
        let ordinal = if is_max {
            member_count.saturating_sub(1) as i64
        } else {
            0
        };
        let builder = &self.state.builder;
        let integer = builder.emit_sol_constant(ordinal, builder.types.ui256, &block);
        let value = builder.emit_sol_enum_cast(integer, result_type, &block);
        Ok((value, block))
    }

    /// `type(I).interfaceId` is the EIP-165 identifier: the XOR of the selectors
    /// of the functions declared directly within interface `I` (inherited
    /// functions excluded, matching solc), as a `bytes4` constant.
    fn emit_type_interface_id(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let Expression::TypeExpression(type_expression) = access.operand() else {
            unreachable!("type(I).interfaceId has a type-expression operand");
        };
        let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name() else {
            unreachable!("type(I).interfaceId names an interface by identifier path");
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
        let builder = &self.state.builder;
        let integer_type = Type::from(IntegerType::unsigned(builder.context, 32));
        let integer = builder.emit_constant(&BigInt::from(interface_id), integer_type, &block);
        let value = builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(4), &block);
        Ok((value, block))
    }
}
