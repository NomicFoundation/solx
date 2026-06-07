//!
//! External / bare-address call lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StateVariableDefinition;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::static_mode::StaticMode;
use crate::ast::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The SOLE `ext_icall` sink for `SelfExternal` + `ExternalInstance`.
    ///
    /// The oracle took a single `&ExternalCall` bundle (a forbidden second
    /// top-level type under §2a); the recut flattens the bundle and enum-izes
    /// `static_call` into [`StaticMode`] (R8-4). At 9 args (`&self` + 8) this is
    /// the one signature above the `clippy.toml` `too-many-arguments-threshold`;
    /// the fill bundles a frozen param struct or splits the receiver (R8-4) — NO
    /// `#[allow]` (Rule 11). It is a deliberate WARN at the skeleton tip.
    pub fn emit_external_call(
        &self,
        receiver: Value<'context, 'block>,
        selector: u32,
        parameter_types: &[Type<'context>],
        return_types: &[Type<'context>],
        argument_values: &[Value<'context, 'block>],
        call_value: Option<Value<'context, 'block>>,
        static_mode: StaticMode,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>> {
        let builder = &self.expression_emitter.state.builder;
        // The receiver is cast to an address and packed with the selector into
        // an external function reference; the call value defaults to zero wei.
        let address = builder.emit_sol_address_cast(receiver, builder.types.sol_address, block);
        let ext_func_ref_type = builder.types.ext_func_ref(parameter_types, return_types);
        let callee =
            builder.emit_sol_ext_func_constant(address, selector, ext_func_ref_type, block);
        let value =
            call_value.unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, block));
        builder.emit_sol_ext_icall(
            callee,
            argument_values,
            return_types,
            value,
            matches!(static_mode, StaticMode::Static),
            block,
        )
    }

    /// Maps the callee's state mutability to its external-call mode: a `view` or
    /// `pure` function lowers to a `STATICCALL`; anything else a normal `CALL`.
    fn static_mode(function_definition: &FunctionDefinition) -> StaticMode {
        match function_definition.mutability() {
            FunctionMutability::View | FunctionMutability::Pure => StaticMode::Static,
            _ => StaticMode::Call,
        }
    }

    /// Emits a self getter call (`this.v(args)`); A4 (#H-M7): nested/reference
    /// getters are a LOUD residual here.
    pub fn emit_self_getter_call(
        &self,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (access, state_variable, arguments, call_value, block);
        unimplemented!("self getter call")
    }

    /// Emits an external getter call (`instance.value()` scalar); A4
    /// (#H-M10/M11): arg-bearing mapping/array getters are a LOUD residual.
    pub fn emit_external_getter_call(
        &self,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (access, state_variable, arguments, block);
        unimplemented!("external getter call")
    }

    /// Emits a bare address call (`addr.call`/`delegatecall`/`staticcall`),
    /// returning `(success, returndata-pointer, block)`. Inner
    /// `_ => unreachable!("bare call kind must be Call/Delegatecall/Staticcall")`.
    pub fn emit_bare_call(
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
        let _ = (access, kind, arguments, call_value, block);
        unimplemented!("bare address call")
    }

    /// Emits a bare address call in result-binding position
    /// (`(ok, data) = addr.call{..}(data)`).
    pub fn emit_bare_call_results(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let _ = (access, kind, call_value, arguments, block);
        unimplemented!("bare address call results")
    }

    /// Emits a multi-result external call (`(a, b) = recv.f(args)`); always a
    /// `sol.ext_icall`.
    pub fn emit_external_call_results(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let selector = function_definition
            .compute_selector()
            .expect("an external call resolves to a function with a selector");
        // The signature comes from the callee's definition (valid for both a
        // foreign instance and the current contract's own function), so the
        // unified path never depends on the callee being in the local registry.
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            function_definition,
            &self.expression_emitter.state.builder,
        );
        // The receiver is the member operand: `this` for a self call, the
        // instance value for an external one — both evaluate to an address.
        let (receiver, current_block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let (argument_values, current_block) =
            self.emit_coerced_arguments(arguments, &parameter_types, current_block)?;
        let results = self.emit_external_call(
            receiver,
            selector,
            &parameter_types,
            &return_types,
            &argument_values,
            call_value,
            Self::static_mode(function_definition),
            &current_block,
        )?;
        Ok((results, current_block))
    }

    /// Recognises and emits an external call in `try` position; `None` = "not a
    /// try-lowerable external-call shape" (a normal outcome).
    pub fn emit_external_call_try(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            Value<'context, 'block>,
            Vec<Value<'context, 'block>>,
            BlockRef<'context, 'block>,
        )>,
    > {
        let _ = (call, block);
        unimplemented!("external call try")
    }
}
