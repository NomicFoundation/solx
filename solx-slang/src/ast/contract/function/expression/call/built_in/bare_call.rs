//!
//! Low-level bare calls — `addr.call` / `addr.delegatecall` / `addr.staticcall`
//! (the `BareCall*` ops) — plus the tuple-result deconstruction paths for both
//! bare calls (`(ok, data) = addr.call(...)`) and genuine external member calls
//! (`(a, b) = recv.f(args)`).
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits one of the bare-call ops and returns both `(status, ret_data)`
    /// SSA values. Gas defaults to `gasleft()`; value defaults to zero for
    /// `addr.call`. Call options (`{gas: g, value: v}`) are not yet handled.
    pub(crate) fn emit_bare_call(
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
        let (addr, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (input_values, block) = self.emit_argument_values(arguments, block)?;
        let input = input_values[0];

        let builder = &self.expression_emitter.state.builder;
        // The bare-call data buffer must live in memory; a `bytes` argument
        // sourced from storage/calldata (`addr.call(savedData)`) is copied into
        // memory first (`sol.bare_call`'s `inp` rejects a non-memory operand).
        let input = TypeConversion::from_target_type(builder.types.sol_string_memory, builder)
            .emit(input, builder, &block);
        let gas = block
            .append_operation(
                GasLeftOperation::builder(builder.context, builder.unknown_location)
                    .val(builder.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("gasleft always produces one result")
            .into();

        let operation: Operation = match kind {
            BuiltIn::AddressCall => {
                // `addr.call{value: v}(data)` forwards `v` as the CALL value;
                // a plain `addr.call(data)` (no options) sends zero. Delegate-
                // and staticcall cannot carry value, so they ignore it.
                let val = call_value
                    .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &block));
                BareCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .val(val)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            BuiltIn::AddressDelegatecall => {
                BareDelegateCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            BuiltIn::AddressStaticcall => {
                BareStaticCallOperation::builder(builder.context, builder.unknown_location)
                    .addr(addr)
                    .gas(gas)
                    .inp(input)
                    .status(builder.types.i1)
                    .ret_data(builder.types.sol_string_memory)
                    .build()
                    .into()
            }
            _ => unreachable!("bare call kind must be Call, Delegatecall, or Staticcall"),
        };

        let result = block.append_operation(operation);
        let status = result
            .result(0)
            .expect("bare call always produces a status")
            .into();
        let ret_data = result
            .result(1)
            .expect("bare call always produces return data")
            .into();
        Ok((status, ret_data, block))
    }

    /// Resolves the low-level bare-call kind for a member-access callee
    /// (`recv.call` / `recv.delegatecall` / `recv.staticcall`), or `None` if
    /// the member is not a low-level call.
    ///
    /// slang resolves these to a built-in for a plain `address` receiver, but
    /// fails to type a library-as-address receiver (`address(L).delegatecall`),
    /// leaving the member unresolved. In that case fall back to the member
    /// name — it is reserved for address low-level calls, and a user method of
    /// the same name would resolve to a `Definition`, so the name fallback only
    /// fires for a member that resolves to nothing at all.
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

    /// Tries to emit a multi-result bare call (`addr.call`, `addr.delegatecall`,
    /// or `addr.staticcall`) used as the right-hand side of tuple
    /// deconstruction. Returns `Ok(None)` if the callee is not a bare-call
    /// member access so the caller can fall through to the named-function
    /// dispatch path.
    pub fn try_emit_bare_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        // `(bool, bytes) = recv.call{value: v}(data)` — peel an optional
        // `{value: v}` call-options layer and forward `v` as the CALL value; a
        // plain `recv.call(data)` carries no options.
        //
        // Resolve the bare-call KIND from the inner member access *before*
        // emitting anything: a non-bare-call callee (e.g. an external
        // `ins.f{value: x, gas: g()}(a)`) must leave the block untouched for the
        // caller's fallback. Emitting the option values here would double-
        // evaluate their side effects and corrupt the observable order.
        let (access, options) = match call.operand() {
            Expression::MemberAccessExpression(access) => (access, None),
            Expression::CallOptionsExpression(call_options) => match call_options.operand() {
                Expression::MemberAccessExpression(access) => (access, Some(call_options)),
                // Not a bare-call shape → not applicable, caller falls back.
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        let Some(kind) = Self::resolve_bare_call_kind(&access) else {
            return Ok(None);
        };

        // Committed to the bare call: now evaluate the options (`{value: v}`
        // captured as the CALL value, others for side effects only).
        let mut current_block = block;
        let mut call_value = None;
        if let Some(call_options) = options {
            (call_value, current_block) = self.capture_call_value(&call_options, current_block)?;
        }
        let (status, ret_data, block) =
            self.emit_bare_call(&access, kind, arguments, call_value, current_block)?;
        Ok(Some((vec![status, ret_data], block)))
    }

    /// Tries to emit an external member call `recv.f(args)` / `this.f(args)`
    /// used as the right-hand side of tuple deconstruction, returning every
    /// decoded result value in declaration order. Returns `Ok(None)` if the
    /// callee is not a member access resolving to a function, so the caller
    /// can fall through to the named-function dispatch path.
    ///
    /// Library and bare-call (`addr.call` / `delegatecall` / `staticcall`)
    /// callees are handled by earlier branches of
    /// [`Self::emit_function_call_results`], so this path only ever sees genuine
    /// external contract calls. The call is lowered as a real `sol.ext_icall`,
    /// which is always correct for tuple returns even when a same-contract
    /// `this.f()` could otherwise use the local-call shortcut.
    pub fn try_emit_external_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        // Unwrap an optional `{value: v}` / `{gas: g}` call-options layer. Only
        // the value is forwarded; gas is left to the backend's default stipend.
        let mut current_block = block;
        let mut call_value: Option<Value<'context, 'block>> = None;
        let access = match call.operand() {
            Expression::MemberAccessExpression(access) => access,
            Expression::CallOptionsExpression(options) => {
                (call_value, current_block) = self.capture_call_value(&options, current_block)?;
                match options.operand() {
                    Expression::MemberAccessExpression(access) => access,
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };
        let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
            access.member().resolve_to_definition()
        else {
            return Ok(None);
        };
        let Some(selector) = function_definition.compute_selector() else {
            return Ok(None);
        };
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            &function_definition,
            &self.expression_emitter.state.builder,
        );

        let (receiver_value, next) =
            self.expression_emitter.emit_value(&access.operand(), current_block)?;
        current_block = next;
        let mut argument_values = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        let builder = &self.expression_emitter.state.builder;
        self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
        let address = builder.emit_sol_address_cast(
            receiver_value,
            builder.types.sol_address,
            &current_block,
        );
        let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
        let callee_ref =
            builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &current_block);
        let value = call_value
            .unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &current_block));
        // A call to a `view`/`pure` function uses `STATICCALL`: the static type
        // of the callee determines this (e.g. calling through a `view` interface
        // method), so it reverts if the callee mutates state, matching solc.
        let static_call = is_static_call_mutability(&function_definition);
        let results = builder.emit_sol_ext_icall(
            callee_ref,
            &argument_values,
            &return_types,
            value,
            static_call,
            &current_block,
        )?;
        Ok(Some((results, current_block)))
    }
}
