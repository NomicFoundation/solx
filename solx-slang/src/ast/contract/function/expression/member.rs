//!
//! Member access expression emission: `base.member`. Routes a namespace-
//! qualified state-variable / constant read, a struct field read, and a
//! built-in member access; the struct-field address routine is shared with the
//! lvalue write path.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::EmitAddress;
use crate::ast::Place;
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
            .expect("slang validates the accessed field is a struct member");

        let BlockAnd {
            value: base_value,
            block,
        } = base.emit(context, block);
        let builder = &context.state.builder;

        let index_value = crate::ast::Value::constant(
            field_index as i64,
            crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
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
                    .expect("a Solidity constant has an initializer");
                return initializer.emit(context, block);
            }
            _ => {}
        }
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
        let value = crate::ast::Pointer::new(address).load(
            crate::ast::Type::new(element_type),
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
