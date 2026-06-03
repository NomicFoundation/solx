//!
//! Member access expression lowering for struct fields: `s.field`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers `s.field` for a struct base to `sol.gep`, followed by a
    /// `sol.load` of the addressed field unless the field already IS the
    /// value (non-ptr-ref-in-storage rule).
    ///
    /// Returns `Ok(None)` when the base is not a struct (e.g. `msg.sender`),
    /// so the caller can fall back to built-in member access lowering.
    pub fn emit_struct_field(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        let Some((address, element_type, block)) = self.emit_struct_field_address(access, block)?
        else {
            return Ok(None);
        };
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        Ok(Some((value, block)))
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            Value<'context, 'block>,
            Type<'context>,
            BlockRef<'context, 'block>,
        )>,
    > {
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
            .expect("unknown struct member");

        let (base_value, block) = self.emit_value(&base, block)?;
        let builder = &self.state.builder;

        let index_value = builder.emit_sol_constant(field_index as i64, builder.types.ui64, &block);
        // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
        // `sol::getEltType` on the C++ side.
        let element_type = unsafe {
            Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                base_value.r#type().to_raw(),
                field_index as u64,
            ))
        };
        let address = builder.emit_sol_gep(base_value, index_value, element_type, &block);
        Ok(Some((address, element_type, block)))
    }
}
