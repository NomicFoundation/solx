//!
//! Member access expression lowering for struct fields: `s.field`.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context> ExpressionEmitter<'state, 'context> {
    /// Lowers `s.field` for a struct base to `sol.gep`, followed by a
    /// `sol.load` of the addressed field unless the field already IS the
    /// value (non-ptr-ref-in-storage rule).
    ///
    /// Returns `Ok(None)` when the base is not a struct, so the caller can
    /// fall back to built-in member access lowering.
    pub fn emit_struct_field(
        &self,
        access: &MemberAccessExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        let Some((address, element_type)) = self.emit_struct_field_address(access, context)? else {
            return Ok(None);
        };
        let value = address.load(element_type, context);
        Ok(Some(value))
    }

    /// Emits the address yielded by `s.field` together with the field's
    /// element MLIR type, without the trailing `sol.load`.
    ///
    /// Shared between the value-producing read path
    /// ([`Self::emit_struct_field`]) and the lvalue write path in
    /// `emit_assignment`. Returns `Ok(None)` when the base is not a struct.
    pub fn emit_struct_field_address(
        &self,
        access: &MemberAccessExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<(Place<'context>, Type<'context>)>> {
        let base = access.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            return Ok(None);
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        let member_name = access.member().name();
        let field_index = struct_definition
            .members()
            .iter()
            .position(|member| member.name().name() == member_name)
            .ok_or_else(|| anyhow::anyhow!("unknown struct member: {member_name}"))?;

        let base_value = self.emit_value(&base, context)?;

        let index_value = Value::constant(
            field_index as i64,
            Type::unsigned(context.melior, solx_utils::BIT_LENGTH_X64),
            context,
        );
        let element_type = base_value.r#type().element_type(field_index as u64);
        let address = Place::from(base_value).gep(index_value, element_type, context);
        Ok(Some((address, element_type)))
    }
}
