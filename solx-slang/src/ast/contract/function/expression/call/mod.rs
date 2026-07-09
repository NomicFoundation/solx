//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use anyhow::Context as _;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;

use solx_mlir::Context;
use solx_mlir::Function;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;

use self::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context>,
}

impl<'emitter, 'state, 'context> CallEmitter<'emitter, 'state, 'context> {
    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Handles type conversions and built-in dispatch, then resolves
    /// user-defined callees through slang's binder to a function definition
    /// node id and looks up the registered MLIR signature.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is unsupported, arguments contain
    /// unsupported constructs, or the callee does not resolve to a registered
    /// function definition.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Option<Value<'context>>> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        let callee = call.operand();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
            let value = self.expression_emitter.emit_value(&first, context)?;

            let target_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type(), context)
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

            let result =
                TypeConversion::from_target_type(target_type, context).emit(value, context);
            return Ok(Some(result));
        }

        if let Some(value) = self.try_emit_built_in_call(&callee, positional_arguments, context)? {
            return Ok(value);
        }

        if let Some(value) =
            self.try_emit_built_in_call_expression(call, positional_arguments, context)?
        {
            return Ok(Some(value));
        }

        if let Expression::MemberAccessExpression(access) = &callee {
            return self.emit_built_in_member_access(access, Some(positional_arguments), context);
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            anyhow::bail!("unsupported callee expression");
        };
        let function_definition = match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => function_definition,
            Some(Definition::Struct(struct_definition)) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type(), context)
                    .ok_or_else(|| anyhow::anyhow!("unresolved struct constructor type"))?;
                return self
                    .emit_struct_constructor(
                        &struct_definition,
                        result_type,
                        positional_arguments,
                        context,
                    )
                    .map(Some);
            }
            _ => anyhow::bail!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            ),
        };

        let results = self
            .emit_direct_call(&function_definition, positional_arguments, context)
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))?;
        Ok(results.into_iter().next())
    }

    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        positional_arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let struct_address = Place::malloc(result_type, context);

        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(positional_arguments.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                context,
            );
            let index_value = Value::constant(
                index as i64,
                Type::unsigned(context.melior, solx_utils::BIT_LENGTH_X64),
                context,
            );
            let field_address = struct_address.gep(index_value, field_type, context);

            let argument_value = self.expression_emitter.emit_value(&argument, context)?;
            let stored =
                TypeConversion::from_target_type(field_type, context).emit(argument_value, context);
            field_address.store(stored, context);
        }

        Ok(struct_address.into())
    }

    /// Emits a direct, named function call and returns all of its result
    /// values in declaration order.
    ///
    /// Unlike [`Self::emit_function_call`], this entry point does not handle
    /// explicit type conversions or built-in dispatch; it is intended for
    /// callers that need the full result tuple.
    ///
    /// # Errors
    ///
    /// Returns an error if the call uses non-positional arguments, if the
    /// callee is not a named identifier, or if name resolution fails.
    pub fn emit_function_call_results(
        &self,
        call: &FunctionCallExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Vec<Value<'context>>> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        let Expression::Identifier(callee_identifier) = call.operand() else {
            anyhow::bail!("multi-result calls only support direct named function callees");
        };
        let Some(Definition::Function(function_definition)) =
            callee_identifier.resolve_to_definition()
        else {
            anyhow::bail!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            );
        };

        self.emit_direct_call(&function_definition, positional_arguments, context)
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))
    }

    /// Emits each argument, resolves the callee's MLIR signature, casts each
    /// argument to its declared parameter type, and emits the `sol.call`,
    /// returning all result values in declaration order.
    fn emit_direct_call(
        &self,
        function_definition: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Vec<Value<'context>>> {
        let mut argument_values = Vec::new();
        for argument in positional_arguments.iter() {
            let value = self.expression_emitter.emit_value(&argument, context)?;
            argument_values.push(value);
        }

        let (mlir_name, parameter_types, return_types) =
            context.resolve_function(function_definition.node_id())?;

        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            *value =
                TypeConversion::from_target_type(parameter_type, context).emit(*value, context);
        }

        Function::call(mlir_name, &argument_values, return_types, context)
    }

    /// Emits a bare member access expression.
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let value = self.emit_built_in_member_access(access, None, context)?;
        Ok(value.expect("bare member access always produces a value"))
    }
}
