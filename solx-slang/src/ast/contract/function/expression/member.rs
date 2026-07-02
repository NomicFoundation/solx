//!
//! Member access expression lowering for struct fields: `s.field`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_place::EmitPlace;
use crate::ast::place::Place;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for MemberAccessExpression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

    /// Lowers `s.field` for a struct base to `sol.gep` + `sol.load`, or falls back to a built-in
    /// member access (e.g. `msg.sender`) when the base is not a struct.
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
