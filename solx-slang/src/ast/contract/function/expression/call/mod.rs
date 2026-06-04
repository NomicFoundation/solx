//!
//! Function call and member access expression lowering.
//!

/// Built-in function and member lowering.
pub mod built_in;
/// Struct-literal constructor lowering.
pub mod struct_constructor;
/// Type conversion and Slang-to-MLIR type resolution.
pub mod type_conversion;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    pub expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Handles explicit type conversions and built-in dispatch, then resolves
    /// user-defined callees through the binder to a function (or struct
    /// constructor) and emits a `sol.call`.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("non-positional call arguments are not yet supported");
        };

        let callee = call.operand();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
            let (value, block) = self.expression_emitter.emit_value(&first, block)?;
            let builder = &self.expression_emitter.state.builder;
            let target_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type())
                .expect("the binder types every type-conversion call");
            let result =
                TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
            return Ok((Some(result), block));
        }

        if let Some((value, block)) =
            self.try_emit_built_in_call(&callee, positional_arguments, block)?
        {
            return Ok((value, block));
        }

        if let Some((value, block)) =
            self.try_emit_built_in_call_expression(call, positional_arguments, block)?
        {
            return Ok((Some(value), block));
        }

        if let Expression::MemberAccessExpression(access) = &callee {
            return self.emit_built_in_member_access(access, Some(positional_arguments), block);
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            unimplemented!("call to this kind of callee is not yet supported");
        };
        let function_definition = match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => function_definition,
            Some(Definition::Struct(struct_definition)) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("the binder types every struct constructor call");
                return self
                    .emit_struct_constructor(
                        &struct_definition,
                        result_type,
                        positional_arguments,
                        block,
                    )
                    .map(|(value, block)| (Some(value), block));
            }
            _ => unimplemented!(
                "call to '{}' is not yet supported",
                callee_identifier.name()
            ),
        };

        let (mlir_name, argument_values, return_types, current_block) = self
            .emit_call_setup(&function_definition, positional_arguments, block)
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))?;

        if return_types.is_empty() {
            self.expression_emitter.state.builder.emit_sol_call(
                mlir_name,
                &argument_values,
                &[],
                &current_block,
            )?;
            Ok((None, current_block))
        } else {
            let result = self
                .expression_emitter
                .state
                .builder
                .emit_sol_call(mlir_name, &argument_values, return_types, &current_block)?
                .expect("a function call with return types always produces a result");
            Ok((Some(result), current_block))
        }
    }

    /// Emits a direct, named function call and returns all of its result
    /// values in declaration order.
    ///
    /// Unlike [`Self::emit_function_call`], this entry point does not handle
    /// explicit type conversions or built-in dispatch — it is intended for
    /// callers that need the full result tuple (e.g. tuple deconstruction).
    pub fn emit_function_call_results(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("non-positional call arguments are not yet supported");
        };

        let Expression::Identifier(callee_identifier) = call.operand() else {
            unimplemented!("multi-result calls only support direct named function callees");
        };
        let Some(Definition::Function(function_definition)) =
            callee_identifier.resolve_to_definition()
        else {
            unimplemented!(
                "call to '{}' is not yet supported in multi-result position",
                callee_identifier.name()
            );
        };

        let (mlir_name, argument_values, return_types, current_block) = self
            .emit_call_setup(&function_definition, positional_arguments, block)
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))?;

        let results = self
            .expression_emitter
            .state
            .builder
            .emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
        Ok((results, current_block))
    }

    /// Emits argument values for a named call, resolves the callee's MLIR
    /// signature, and casts each argument to its declared parameter type.
    ///
    /// Returns the resolved MLIR name, the cast argument values, the declared
    /// return types, and the block in which the call should be emitted.
    fn emit_call_setup<'a>(
        &'a self,
        function_definition: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [melior::ir::Type<'context>],
        BlockRef<'context, 'block>,
    )> {
        let mut argument_values = Vec::new();
        let mut current_block = block;
        for argument in positional_arguments.iter() {
            let (value, next_block) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id())?;

        let builder = &self.expression_emitter.state.builder;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, builder);
            *value = conversion.emit(*value, builder, &current_block);
        }

        Ok((mlir_name, argument_values, return_types, current_block))
    }

    /// Emits a bare member access expression (e.g. `tx.origin`, `msg.sender`).
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_built_in_member_access(access, None, block)?;
        Ok((
            value.expect("a bare member access always produces a value"),
            block,
        ))
    }
}
