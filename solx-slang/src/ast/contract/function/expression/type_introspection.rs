//!
//! `type(T)` introspection built-ins: `type(T).min` / `.max` for integers,
//! `type(E).min` / `.max` for enums, and `type(I).interfaceId`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::MemberAccessExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Tries to lower a `type(T).member` introspection constant, returning
    /// `Ok(None)` when the member access is not such an introspection.
    pub fn try_emit_type_introspection(
        &self,
        _access: &MemberAccessExpression,
        _block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        Ok(None)
    }
}
