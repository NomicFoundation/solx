//!
//! Member access expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::MemberAccessExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a member access `operand.member`.
    ///
    /// A struct-field access (`s.field`) is tried first; otherwise the member
    /// is an EVM built-in — the environment globals (`msg.*`, `tx.*`,
    /// `block.*`), or an operand-bearing member (`address.balance` /
    /// `.codehash` / `.code`, `x.length`). Namespace-qualified reads, enum
    /// variants, and selectors defer to later domains.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Some(result) = self.emit_struct_field(access, block)? {
            return Ok(result);
        }
        if let Some(result) = self.try_emit_type_introspection(access, block)? {
            return Ok(result);
        }
        match access.member().resolve_to_built_in() {
            Some(
                built_in @ (BuiltIn::AddressBalance
                | BuiltIn::AddressCodehash
                | BuiltIn::AddressCode
                | BuiltIn::Length),
            ) => self.emit_unary_member(built_in, access, block),
            Some(built_in) => Ok((
                self.emit_environment_global(built_in, access, &block),
                block,
            )),
            None => unimplemented!("member access lowering: {}", access.member().name()),
        }
    }

    /// Lowers a struct-field read `s.field`, returning `Ok(None)` when the base
    /// is not a struct so the caller falls back to built-in member access.
    fn emit_struct_field(
        &self,
        _access: &MemberAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        unimplemented!("member access: struct field")
    }

    /// Lowers a nullary environment global (`msg.*`, `tx.*`, `block.*`) to its
    /// `sol.*` intrinsic.
    fn emit_environment_global(
        &self,
        _built_in: BuiltIn,
        _access: &MemberAccessExpression,
        _block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        unimplemented!("member access: environment global")
    }

    /// Lowers an operand-bearing member intrinsic (`address.balance` /
    /// `.codehash` / `.code`, `x.length`).
    fn emit_unary_member(
        &self,
        _built_in: BuiltIn,
        _access: &MemberAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        unimplemented!("member access: unary member")
    }
}
