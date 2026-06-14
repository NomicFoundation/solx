//!
//! Member access expression lowering: `base.member`. Routes a namespace-
//! qualified state-variable / constant read, a struct field read, and a
//! built-in member access; the struct-field address helper is shared with the
//! lvalue write path.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Lowers `s.field` for a struct base to `sol.gep`, followed by a
    /// `sol.load` of the addressed field unless the field already IS the
    /// value (non-ptr-ref-in-storage rule).
    ///
    /// The caller is responsible for routing only struct-base member accesses
    /// here; non-struct bases (e.g. `msg.sender`) go to built-in member access
    /// lowering.
    pub fn emit_struct_field(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (address, element_type, block) = self.emit_struct_field_address(access, block)?;
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        Ok((value, block))
    }

    /// Emits the address yielded by `s.field` together with the field's
    /// element MLIR type, without the trailing `sol.load`.
    ///
    /// Shared between the value-producing read path
    /// ([`Self::emit_struct_field`]) and the lvalue write path in
    /// `emit_assignment`. Only called for a struct base.
    pub fn emit_struct_field_address(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Type<'context>,
        BlockRef<'context, 'block>,
    )> {
        let base = access.operand();
        let Some(SlangType::Struct(struct_type)) = base.get_type() else {
            unreachable!("emit_struct_field_address is only called for a struct base");
        };
        let Definition::Struct(struct_definition) = struct_type.definition() else {
            unreachable!("slang StructType always references a Struct definition");
        };

        // Resolve the accessed field to its `StructMember` definition and locate
        // it by node-id identity — slang exposes struct fields as an ordered list
        // with no direct field-index lookup, but the binder resolves the access,
        // so no name-string comparison is needed (Rule-7).
        let Some(Definition::StructMember(member_definition)) =
            access.member().resolve_to_definition()
        else {
            unreachable!("slang resolves a struct field access to its StructMember definition");
        };
        let member_id = member_definition.node_id();
        let field_index = struct_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_id)
            .expect("slang validates the accessed field is a struct member");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(self, block)?;
        let builder = &self.state.builder;

        let index_value = builder.emit_sol_constant(
            field_index as i64,
            crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_X64).into_mlir(),
            &block,
        );
        // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
        // `sol::getEltType` on the C++ side.
        let element_type = unsafe {
            Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                base_value.r#type().to_raw(),
                field_index as u64,
            ))
        };
        let address =
            builder.emit_sol_gep(base_value.into_mlir(), index_value, element_type, &block);
        Ok((address, element_type, block))
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
                return context
                    .emit_state_variable_read(&state_variable, block)
                    .map(|(value, block)| BlockAnd {
                        block,
                        value: value.into(),
                    });
            }
            Some(Definition::Constant(constant)) => {
                let initializer = constant
                    .value()
                    .expect("a Solidity constant has an initializer");
                return initializer.emit(context, block);
            }
            _ => {}
        }
    }
    // A struct-typed base is a field read (`s.field`); anything else
    // (e.g. `msg.sender`, `addr.balance`) is a built-in member access.
    if matches!(node.operand().get_type(), Some(SlangType::Struct(_))) {
        context
            .emit_struct_field(node, block)
            .map(|(value, block)| BlockAnd {
                block,
                value: value.into(),
            })
    } else {
        // `msg.sender`, `addr.balance`, `arr.length`: a built-in member access,
        // which in value position always yields a value.
        let (value, block) = context.emit_built_in_member_access(node, None, block)?;
        Ok(BlockAnd {
            block,
            value: value.expect("a bare member access yields a value").into(),
        })
    }
});
