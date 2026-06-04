//!
//! External member calls — `recv.f(args)`, `this.f(args)`, `I(addr).f(args)`.
//!
//! A call whose member resolves to a selector-carrying function and whose
//! receiver is a contract / interface / address value is lowered as a real
//! `sol.ext_icall` (CALL, or STATICCALL for a `view`/`pure` callee). A
//! same-contract `this.f()` is also lowered this way — it is a genuine
//! external call, not a local jump.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to emit an external member call `recv.f(args)` / `this.f(args)`,
    /// returning every decoded result in declaration order (so both the single-
    /// and multi-result dispatch can use it). Returns `Ok(None)` when the
    /// callee is not an external call, so the caller falls through.
    ///
    /// An optional `{value: v}` call-options layer is peeled and forwarded as
    /// the CALL value. The receiver is cast to `address`, an external function
    /// reference is built from the resolved selector and signature, and the
    /// call is a `STATICCALL` when the callee is `view`/`pure`.
    ///
    /// A namespace-qualified callee (`L.f` / `C.f` / `import.f`) and `super.f`
    /// are excluded — those are library / namespace / super dispatch handled by
    /// other domains. The `compute_selector` guard further excludes internal
    /// callees (an internal `using`-for function carries no selector); a public
    /// `using`-for library function on a value receiver is the one shape this
    /// cannot yet distinguish from an external call and would mis-lower, until
    /// the library-call domain claims it ahead of this path.
    pub fn try_emit_external_call_results(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let mut block = block;
        let mut call_value: Option<Value<'context, 'block>> = None;
        let access = match call.operand() {
            Expression::MemberAccessExpression(access) => access,
            Expression::CallOptionsExpression(options) => {
                (call_value, block) = self.capture_call_value(&options, block)?;
                match options.operand() {
                    Expression::MemberAccessExpression(access) => access,
                    _ => return Ok(None),
                }
            }
            _ => return Ok(None),
        };

        // A namespace operand (`L.f` / `C.f` / `import.f`) or `super.f` is not a
        // value-receiver external call.
        let operand = access.operand();
        let namespace_operand = matches!(&operand, Expression::Identifier(identifier)
        if matches!(
            identifier.resolve_to_definition(),
            Some(
                Definition::Library(_)
                    | Definition::Contract(_)
                    | Definition::Import(_)
                    | Definition::ImportedSymbol(_)
            )
        ));
        if namespace_operand || matches!(operand, Expression::SuperKeyword(_)) {
            return Ok(None);
        }
        let Some(Definition::Function(function)) = access.member().resolve_to_definition() else {
            return Ok(None);
        };
        let Some(selector) = function.compute_selector() else {
            return Ok(None);
        };

        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            &function,
            &self.expression_emitter.state.builder,
        );

        let (receiver_value, next) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        block = next;
        let mut argument_values = Vec::with_capacity(arguments.len());
        for argument in arguments.iter() {
            let (value, next) = self.expression_emitter.emit_value(&argument, block)?;
            argument_values.push(value);
            block = next;
        }
        self.coerce_arguments(&mut argument_values, &parameter_types, &block);

        let builder = &self.expression_emitter.state.builder;
        let address =
            builder.emit_sol_address_cast(receiver_value, builder.types.sol_address, &block);
        let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
        let callee_ref =
            builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &block);
        let value =
            call_value.unwrap_or_else(|| builder.emit_sol_constant(0, builder.types.ui256, &block));
        // A call to a `view`/`pure` function lowers to STATICCALL, reverting if
        // the callee mutates state (matching solc).
        let static_call = is_static_call_mutability(&function);
        let results = builder.emit_sol_ext_icall(
            callee_ref,
            &argument_values,
            &return_types,
            value,
            static_call,
            &block,
        )?;
        Ok(Some((results, block)))
    }
}

/// Whether a callee's declared mutability (`view` / `pure`) makes its external
/// call a `STATICCALL`.
fn is_static_call_mutability(function: &FunctionDefinition) -> bool {
    matches!(
        function.mutability(),
        FunctionMutability::View | FunctionMutability::Pure
    )
}
