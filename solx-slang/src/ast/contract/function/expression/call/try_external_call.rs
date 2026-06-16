//!
//! An external call in `try` position, classified ahead of emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::ExtICallOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

/// A `try recv.f(args)` external call, resolved from the `try` expression. Only
/// this shape carries a real catch path; classification is a pure precondition,
/// so [`Self::emit`] is an exact (infallible) emitter rather than an emitter that
/// returns `Option`-as-"not applicable".
pub struct TryExternalCall {
    /// The `{value: v}` / `{gas: g}` options layer, if any (`recv.f{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The `recv.f` member access.
    access: MemberAccessExpression,
    /// The resolved external callee.
    function: FunctionDefinition,
    /// Its ABI selector.
    selector: u32,
    /// The positional call arguments.
    arguments: PositionalArguments,
}

impl TryExternalCall {
    /// A `try` expression is lowerable only when it wraps an external call —
    /// `recv.f(args)`, optionally inside a `{value: v}` / `{gas: g}` call-options
    /// layer. Any other shape yields `None` and runs only the success body.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let Expression::FunctionCallExpression(call) = expression else {
            return None;
        };
        let (options, access) = match call.operand() {
            Expression::MemberAccessExpression(access) => (None, access),
            Expression::CallOptionsExpression(options) => {
                let Expression::MemberAccessExpression(access) = options.operand() else {
                    return None;
                };
                (Some(options), access)
            }
            _ => return None,
        };
        let Some(Definition::Function(function)) = access.member().resolve_to_definition() else {
            return None;
        };
        let selector = function.compute_selector()?;
        let ArgumentsDeclaration::PositionalArguments(arguments) = call.arguments() else {
            return None;
        };
        Some(Self {
            options,
            access,
            function,
            selector,
            arguments,
        })
    }

    /// Emits this external call with `try` semantics, returning the success
    /// status flag, the decoded results, and the continuation block.
    pub fn emit<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        // A `recv.f{value: v}(args)` forwards `v` as msg.value; gas/salt follow the
        // same drop/forward rule as a normal call.
        let mut current_block = block;
        let mut call_value = None;
        if let Some(options) = &self.options {
            let (value, _salt, next_block) = context.capture_call_options(options, current_block);
            current_block = next_block;
            call_value = value;
        }
        // External (ABI) signature: `calldata` reference parameters cross the call
        // boundary as memory (see `resolve_external_function_types`).
        let (parameter_types, return_types) = AstType::resolve_signature(
            &self.function,
            LocationPolicy::ForceMemory,
            &context.state.builder,
        );
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = self.access.operand().emit(context, current_block);
        let (argument_values, current_block) =
            context.emit_coerced_arguments(&self.arguments, &parameter_types, current_block);
        let callee = context.emit_external_callee(
            receiver.into_mlir(),
            self.selector,
            &parameter_types,
            &return_types,
            &current_block,
        );
        let builder = &context.state.builder;
        let value = call_value.unwrap_or_else(|| {
            AstValue::constant(
                0,
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                &current_block,
            )
            .into_mlir()
        });
        // `sol.ext_icall` results are `(i1 status, decoded-returns...)`; the `try`
        // form yields the status instead of reverting on failure, so the caller
        // can run a `catch` handler.
        let mut out_types = Vec::with_capacity(return_types.len() + 1);
        out_types
            .push(AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
        out_types.extend_from_slice(&return_types);
        let operation = current_block.append_operation(sol_op_build!(
            builder,
            ExtICallOperation
                .outs(&out_types)
                .callee(callee)
                .callee_operands(&argument_values)
                .gas(AstValue::gas_left(builder, &current_block))
                .value(value)
                .try_call(Attribute::unit(builder.context))
        ));
        let status = operation
            .result(0)
            .expect("sol.ext_icall try produces a status result")
            .into();
        let results = (0..return_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_icall try produces a status plus its declared results")
                    .into()
            })
            .collect();
        (status, results, current_block)
    }
}
