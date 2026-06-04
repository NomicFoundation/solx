//!
//! Low-level bare calls — `addr.call` / `addr.delegatecall` / `addr.staticcall`.
//!
//! Each yields the `(bool success, bytes memory data)` pair without reverting
//! on failure, so they appear as the right-hand side of a tuple deconstruction
//! (`(bool ok, bytes memory d) = addr.call(payload);`) far more often than in
//! single-value position. Both positions route through
//! [`Self::try_emit_bare_call_results`].
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit a bare call (`addr.call` / `addr.delegatecall` /
    /// `addr.staticcall`), returning its `(status, ret_data)` values for tuple
    /// deconstruction. Returns `Ok(None)` when the callee is not a bare-call
    /// member access, so the caller falls through to the remaining call kinds.
    ///
    /// An optional `{value: v}` call-options layer is peeled and forwarded as
    /// the CALL value (`addr.call{value: v}(data)`). The bare-call kind is
    /// resolved from the inner member access *before* any option or operand is
    /// emitted: a non-bare-call callee must leave the block untouched for the
    /// caller's fallback, since emitting option values here would double-
    /// evaluate their side effects.
    pub fn try_emit_bare_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let (access, options) = match call.operand() {
            Expression::MemberAccessExpression(access) => (access, None),
            Expression::CallOptionsExpression(call_options) => match call_options.operand() {
                Expression::MemberAccessExpression(access) => (access, Some(call_options)),
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        let Some(kind) = Self::resolve_bare_call_kind(&access) else {
            return Ok(None);
        };

        let mut block = block;
        let mut call_value = None;
        if let Some(call_options) = options {
            (call_value, block) = self.capture_call_value(&call_options, block)?;
        }
        let (status, ret_data, block) =
            self.emit_bare_call(&access, kind, arguments, call_value, block)?;
        Ok(Some((vec![status, ret_data], block)))
    }

    /// Emits one of the bare-call ops, returning `(status, ret_data, block)`.
    /// Gas defaults to all remaining gas; the CALL value defaults to zero for
    /// `addr.call`. Delegate- and staticcall carry no value.
    fn emit_bare_call(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (address, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (input_values, block) = self.emit_argument_values(arguments, block)?;

        let builder = &self.expression_emitter.state.builder;
        // The bare-call data buffer must live in memory; a `bytes` argument
        // sourced from storage/calldata (`addr.call(savedData)`) is copied into
        // memory first (the op's `inp` operand rejects a non-memory buffer).
        let input = TypeConversion::from_target_type(builder.types.sol_string_memory, builder)
            .emit(input_values[0], builder, &block);

        let (status, ret_data) = match kind {
            BuiltIn::AddressCall => {
                let value = call_value
                    .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &block));
                builder.emit_sol_bare_call(address, value, input, &block)
            }
            BuiltIn::AddressDelegatecall => {
                builder.emit_sol_bare_delegate_call(address, input, &block)
            }
            BuiltIn::AddressStaticcall => builder.emit_sol_bare_static_call(address, input, &block),
            _ => unreachable!("bare-call kind is Call, Delegatecall, or Staticcall"),
        };
        Ok((status, ret_data, block))
    }

    /// Resolves the bare-call kind for a member-access callee (`recv.call` /
    /// `recv.delegatecall` / `recv.staticcall`), or `None` if the member is not
    /// a low-level call.
    ///
    /// slang resolves these to a built-in for a plain `address` receiver, but
    /// leaves a library-as-address receiver (`address(L).delegatecall`)
    /// unresolved. In that case fall back to the member name — it is reserved
    /// for address low-level calls, and a user method of the same name would
    /// resolve to a `Definition`, so the name fallback only fires for a member
    /// that resolves to nothing at all.
    fn resolve_bare_call_kind(access: &MemberAccessExpression) -> Option<BuiltIn> {
        match access.member().resolve_to_built_in() {
            Some(
                kind @ (BuiltIn::AddressCall
                | BuiltIn::AddressDelegatecall
                | BuiltIn::AddressStaticcall),
            ) => Some(kind),
            Some(_) => None,
            None if access.member().resolve_to_definition().is_none() => {
                match access.member().name().as_str() {
                    "call" => Some(BuiltIn::AddressCall),
                    "delegatecall" => Some(BuiltIn::AddressDelegatecall),
                    "staticcall" => Some(BuiltIn::AddressStaticcall),
                    _ => None,
                }
            }
            None => None,
        }
    }
}
