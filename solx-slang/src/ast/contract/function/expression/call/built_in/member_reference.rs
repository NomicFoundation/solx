//!
//! Member references that resolve to a value rather than a call or EVM
//! intrinsic — currently enum variants (`E.Variant`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Classifies a member access as an enum-variant reference (`E.Variant` or
    /// qualified `C.E.Variant`), returning the variant's ordinal when it is one
    /// (and not a call). The ordinal is located by NodeId identity against the
    /// enum's members, never by comparing the member name as text (Rule-7).
    pub fn enum_variant_ordinal(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
    ) -> Option<usize> {
        if arguments.is_some() {
            return None;
        }
        let Definition::EnumMember(member_definition) = access.member().resolve_to_definition()?
        else {
            return None;
        };
        let Definition::Enum(enum_definition) =
            Self::resolve_member_access_operand(&access.operand())?
        else {
            return None;
        };
        enum_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_definition.node_id())
    }

    /// Emits an enum-variant reference: the variant's `ordinal` as an integer
    /// constant, bridged to the enum type via `sol.enum_cast`.
    pub fn emit_enum_variant(
        &self,
        access: &MemberAccessExpression,
        ordinal: usize,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let result_type = self
            .expression_emitter
            .resolve_slang_type(access.get_type())
            .expect("slang types an enum-variant reference as the enum");
        let builder = &self.expression_emitter.state.builder;
        let raw = builder.emit_sol_constant(ordinal as i64, builder.types.ui256, &block);
        let value = builder.emit_sol_enum_cast(raw, result_type, &block);
        (value, block)
    }

    /// Resolves a member-access operand to its definition: a bare type name
    /// (`E.Variant`, whose operand is the `Identifier` `E`) or a qualified path
    /// whose operand is itself a member access (`C.E.Variant`).
    fn resolve_member_access_operand(operand: &Expression) -> Option<Definition> {
        match operand {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }
}
