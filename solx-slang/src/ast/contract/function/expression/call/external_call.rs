//!
//! External / bare-address call lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StateVariableDefinition;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::static_mode::StaticMode;

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
        let _ = (
            receiver,
            selector,
            parameter_types,
            return_types,
            argument_values,
            call_value,
            static_mode,
            block,
        );
        unimplemented!("external call sink")
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
        let _ = (access, function_definition, call_value, arguments, block);
        unimplemented!("external call results")
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
