//!
//! `assert` and `require` built-in emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use solx_mlir::ods::sol::AssertOperation;
use solx_mlir::ods::sol::RequireOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits an `assert(condition)` built-in via `sol.assert`.
    pub fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let BlockAnd {
            value: condition_value,
            block,
        } = condition.emit(self, block);
        let condition_boolean = condition_value
            .is_nonzero(&self.state.builder, &block)
            .into_mlir();
        sol_op_void!(
            &self.state.builder,
            &block,
            AssertOperation.cond(condition_boolean)
        );
        block
    }

    /// Emits a `require(condition)` or `require(condition, message)` built-in
    /// via `sol.require`.
    ///
    /// Literal string messages lower to `sol.require %cond, "msg" : ()`. A
    /// non-literal expression evaluates at runtime and is ABI-encoded under
    /// the `Error(string)` selector via the `call` form of `sol.require`.
    pub fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let BlockAnd {
            value: condition_value,
            block,
        } = condition.emit(self, block);
        let condition_boolean = condition_value
            .is_nonzero(&self.state.builder, &block)
            .into_mlir();
        let builder = &self.state.builder;
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes).expect("require message is valid UTF-8");
                sol_op_void!(
                    builder,
                    &block,
                    RequireOperation
                        .cond(condition_boolean)
                        .args(&[])
                        .msg(StringAttribute::new(builder.context, &literal))
                );
                block
            }
            Some(Expression::FunctionCallExpression(error_call))
                if Self::call_resolves_to_error(error_call) =>
            {
                self.emit_require_custom_error(condition_boolean, error_call, block)
            }
            Some(expression) => {
                let BlockAnd {
                    value: message_value,
                    block,
                } = expression.emit(self, block);
                let string_memory_type =
                    crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory)
                        .into_mlir();
                let message_value = message_value
                    .coerce_to(crate::ast::Type::new(string_memory_type), builder, &block)
                    .into_mlir();
                sol_op_void!(
                    builder,
                    &block,
                    RequireOperation
                        .cond(condition_boolean)
                        .args(&[message_value])
                        .msg(StringAttribute::new(builder.context, "Error(string)"))
                        .call(Attribute::unit(builder.context))
                );
                block
            }
            None => {
                sol_op_void!(
                    builder,
                    &block,
                    RequireOperation.cond(condition_boolean).args(&[])
                );
                block
            }
        }
    }

    /// Emits `require(condition, CustomError(args))` (Solidity ≥ 0.8.26) as the
    /// `call` form of `sol.require` carrying the error's canonical signature and
    /// its ABI-encoded arguments — the same payload `revert CustomError(args)`
    /// builds, but guarded by the condition.
    fn emit_require_custom_error(
        &self,
        condition: Value<'context, 'block>,
        error_call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let Some(Definition::Error(error_definition)) = Self::callee_definition(error_call) else {
            unreachable!("a require custom error resolves to an error definition");
        };
        let signature = error_definition
            .compute_canonical_signature()
            .expect("slang computes a canonical signature for an error");
        let parameters = error_definition.parameters();
        let ArgumentsDeclaration::PositionalArguments(positional) = error_call.arguments() else {
            unimplemented!("named arguments in a require custom error are not yet supported");
        };
        let mut current_block = block;
        let mut argument_values = Vec::new();
        for argument in positional.iter() {
            let BlockAnd {
                value,
                block: next_block,
            } = argument.emit(self, current_block);
            current_block = next_block;
            argument_values.push(value);
        }
        let builder = &self.state.builder;
        let argument_values: Vec<_> = argument_values
            .into_iter()
            .zip(parameters.iter())
            .map(|(value, parameter)| {
                let parameter_type = crate::ast::Type::resolve(
                    &parameter
                        .get_type()
                        .expect("error parameter type resolved by semantic analysis"),
                    LocationPolicy::Declared(None),
                    builder,
                );
                value
                    .coerce_to(
                        crate::ast::Type::new(parameter_type),
                        builder,
                        &current_block,
                    )
                    .into_mlir()
            })
            .collect();
        sol_op_void!(
            builder,
            &current_block,
            RequireOperation
                .cond(condition)
                .args(&argument_values)
                .msg(StringAttribute::new(builder.context, &signature))
                .call(Attribute::unit(builder.context))
        );
        current_block
    }

    /// Whether `error_call`'s callee resolves to an error definition (a custom
    /// error constructor used as a `require` message), located by typed
    /// resolution rather than by comparing the callee name as text (Rule-7).
    fn call_resolves_to_error(error_call: &FunctionCallExpression) -> bool {
        matches!(
            Self::callee_definition(error_call),
            Some(Definition::Error(_))
        )
    }

    /// Resolves a call's callee to its definition: a bare error name
    /// (`CustomError(...)`) or a qualified one (`Lib.CustomError(...)`).
    fn callee_definition(error_call: &FunctionCallExpression) -> Option<Definition> {
        match error_call.operand() {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => None,
        }
    }
}
