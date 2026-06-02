//!
//! Member-access call dispatch: `this.v(args)` public-getter self-calls,
//! `this.f(args)` external self-calls, same-contract local calls, external
//! instance calls (`I(addr).f(args)` / `instance.f(args)`), and external
//! public-getter calls (`instance.value()`).
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// `this.v(args)` — an external call to a public state variable's
    /// auto-generated getter (`v` a mapping/array/scalar). Like `this.f(args)`
    /// it CALLs the contract's own address with the getter's selector; the
    /// signature is reconstructed from the variable (single-level shapes).
    pub(crate) fn try_emit_this_getter_call(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(arguments) = arguments
            && matches!(access.operand(), Expression::ThisKeyword(_))
            && let Some(Definition::StateVariable(state_variable)) =
                access.member().resolve_to_definition()
            && let Some(selector) = state_variable.compute_selector()
            && let Some((parameter_types, return_types)) = self.getter_signature(&state_variable)
        {
            let mut argument_values = Vec::with_capacity(arguments.len());
            let mut current_block = block;
            for argument in arguments.iter() {
                let (value, next) = self
                    .expression_emitter
                    .emit_value(&argument, current_block)?;
                argument_values.push(value);
                current_block = next;
            }
            let builder = &self.expression_emitter.state.builder;
            self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
            let contract_type = self
                .expression_emitter
                .state
                .current_contract_type
                .ok_or_else(|| anyhow::anyhow!("sol.this emitted outside a contract"))?;
            let this_value = current_block
                .append_operation(
                    ThisOperation::builder(builder.context, builder.unknown_location)
                        .addr(contract_type)
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.this always produces one result")
                .into();
            let results = self.emit_external_call(
                this_value,
                selector,
                &parameter_types,
                &return_types,
                &argument_values,
                call_value,
                false,
                &current_block,
            )?;
            return Ok(Some((results.into_iter().next(), current_block)));
        }
        Ok(None)
    }

    /// `this.f(args)` is a genuine external call in Solidity (CALL to the
    /// contract's own address), so it populates returndata and runs the
    /// dispatcher. Emit a real `sol.ext_icall` rather than the local-call
    /// shortcut below — tests that inspect `returndatasize()` rely on this.
    pub(crate) fn try_emit_this_external_call(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(arguments) = arguments
            && matches!(access.operand(), slang_solidity_v2::ast::Expression::ThisKeyword(_))
            && let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
                access.member().resolve_to_definition()
            && let Some(selector) = function_definition.compute_selector()
        {
            let resolved = self
                .expression_emitter
                .state
                .resolve_function(function_definition.node_id())
                .ok()
                .map(|(_, params, returns)| (params.to_vec(), returns.to_vec()));
            if let Some((parameter_types, return_types)) = resolved {
                let mut argument_values = Vec::with_capacity(arguments.len());
                let mut current_block = block;
                for argument in arguments.iter() {
                    let (value, next) = self
                        .expression_emitter
                        .emit_value(&argument, current_block)?;
                    argument_values.push(value);
                    current_block = next;
                }
                let builder = &self.expression_emitter.state.builder;
                self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
                // `this` as an address.
                let contract_type = self
                    .expression_emitter
                    .state
                    .current_contract_type
                    .ok_or_else(|| anyhow::anyhow!("sol.this emitted outside a contract"))?;
                let this_value = current_block
                    .append_operation(
                        ThisOperation::builder(builder.context, builder.unknown_location)
                            .addr(contract_type)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.this always produces one result")
                    .into();
                let results = self.emit_external_call(
                    this_value,
                    selector,
                    &parameter_types,
                    &return_types,
                    &argument_values,
                    call_value,
                    is_static_call_mutability(&function_definition),
                    &current_block,
                )?;
                return Ok(Some((results.into_iter().next(), current_block)));
            }
        }
        Ok(None)
    }

    /// Experimental: `this.f(args)` / `b.f(args)` whose member resolves to a
    /// function already registered in the current context is lowered as a
    /// local `sol.call` instead of a true external call. Skips real
    /// external-call semantics (gas stipend, reentrancy guards) but is good
    /// enough for tests whose behaviour does not depend on those.
    pub(crate) fn try_emit_local_call(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(arguments) = arguments
            && let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
                access.member().resolve_to_definition()
        {
            let resolved = self
                .expression_emitter
                .state
                .resolve_function(function_definition.node_id())
                .ok()
                .map(|(name, params, returns)| {
                    (name.to_owned(), params.to_vec(), returns.to_vec())
                });
            if let Some((mlir_name, parameter_types, return_types)) = resolved {
                let mut argument_values = Vec::with_capacity(arguments.len());
                let mut current_block = block;
                for argument in arguments.iter() {
                    let (value, next) = self
                        .expression_emitter
                        .emit_value(&argument, current_block)?;
                    argument_values.push(value);
                    current_block = next;
                }
                let builder = &self.expression_emitter.state.builder;
                self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
                let result = builder.emit_sol_call(
                    &mlir_name,
                    &argument_values,
                    &return_types,
                    &current_block,
                )?;
                return Ok(Some((result, current_block)));
            }
        }
        Ok(None)
    }

    /// External call to another contract/interface instance:
    /// `ICounter(addr).f(args)` / `instance.f(args)` where the member
    /// resolves to a function not defined in (registered for) the current
    /// contract. Evaluate the operand as the target address and emit a
    /// real `sol.ext_icall`.
    pub(crate) fn try_emit_external_instance_call(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(arguments) = arguments
            && let Some(slang_solidity_v2::ast::Definition::Function(function_definition)) =
                access.member().resolve_to_definition()
            && let Some(selector) = function_definition.compute_selector()
        {
            let (parameter_types, return_types) = TypeConversion::resolve_function_types(
                &function_definition,
                &self.expression_emitter.state.builder,
            );
            // Evaluate the receiver expression as the callee address.
            let (receiver_value, mut current_block) = self
                .expression_emitter
                .emit_value(&access.operand(), block)?;
            let mut argument_values = Vec::with_capacity(arguments.len());
            for argument in arguments.iter() {
                let (value, next) = self
                    .expression_emitter
                    .emit_value(&argument, current_block)?;
                argument_values.push(value);
                current_block = next;
            }
            self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);
            let results = self.emit_external_call(
                receiver_value,
                selector,
                &parameter_types,
                &return_types,
                &argument_values,
                call_value,
                is_static_call_mutability(&function_definition),
                &current_block,
            )?;
            return Ok(Some((results.into_iter().next(), current_block)));
        }
        Ok(None)
    }

    /// External call to a public state variable's auto-generated getter:
    /// `instance.value()` where `value` is a scalar `public` state var on
    /// another contract. (Mapping/array getters with key args are not
    /// handled here.)
    pub(crate) fn try_emit_external_getter_call(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(arguments) = arguments
            && arguments.is_empty()
            && let Some(slang_solidity_v2::ast::Definition::StateVariable(state_variable)) =
                access.member().resolve_to_definition()
            && let Some(selector) = state_variable.compute_selector()
            && let Ok(return_type) = TypeConversion::resolve_state_variable_type(
                &state_variable,
                &self.expression_emitter.state.builder,
            )
        {
            let return_types = [return_type];
            let (receiver_value, current_block) = self
                .expression_emitter
                .emit_value(&access.operand(), block)?;
            let results = self.emit_external_call(
                receiver_value,
                selector,
                &[],
                &return_types,
                &[],
                None,
                false,
                &current_block,
            )?;
            return Ok(Some((results.into_iter().next(), current_block)));
        }
        Ok(None)
    }
}
