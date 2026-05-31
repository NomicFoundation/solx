//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;

use self::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // Named-argument struct constructors (`S({field: value, ...})`) are
        // translated by mapping each declared member to the value with the
        // matching name, then delegating to the normal struct-constructor
        // emission path.
        if let ArgumentsDeclaration::NamedArguments(named_arguments) = &call.arguments()
            && let Expression::Identifier(callee_identifier) = call.operand()
            && let Some(Definition::Struct(struct_definition)) =
                callee_identifier.resolve_to_definition()
        {
            let result_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type())
                .ok_or_else(|| anyhow::anyhow!("unresolved struct constructor type"))?;
            return self
                .emit_named_struct_constructor(
                    &struct_definition,
                    result_type,
                    named_arguments,
                    block,
                )
                .map(|(value, block)| (Some(value), block));
        }

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
            let (value, block) = self.expression_emitter.emit_value(&first, block)?;
            let builder = &self.expression_emitter.state.builder;

            let target_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type())
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

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

        // `addr.call{value: v, gas: g}(data)` and friends — unwrap the
        // `CallOptionsExpression` and route to the inner member access /
        // new expression. Options are evaluated for side effects but their
        // values do not yet thread through to the underlying op.
        if let Expression::CallOptionsExpression(call_options) = &callee {
            let inner = call_options.operand();
            if let Expression::MemberAccessExpression(access) = &inner {
                let mut current_block = block;
                for option in call_options.options().iter() {
                    let value_expression = option.value();
                    let (_value, next) = self
                        .expression_emitter
                        .emit_value(&value_expression, current_block)?;
                    current_block = next;
                }
                return self.emit_built_in_member_access(
                    access,
                    Some(positional_arguments),
                    current_block,
                );
            }
            if let Expression::NewExpression(_) = &inner {
                let mut current_block = block;
                for option in call_options.options().iter() {
                    let value_expression = option.value();
                    let (_value, next) = self
                        .expression_emitter
                        .emit_value(&value_expression, current_block)?;
                    current_block = next;
                }
                return self.emit_new(call, positional_arguments, current_block);
            }
        }

        if let Expression::NewExpression(_) = &callee {
            return self.emit_new(call, positional_arguments, block);
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            anyhow::bail!("unsupported callee expression");
        };
        let function_definition = match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => function_definition,
            Some(Definition::Struct(struct_definition)) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .ok_or_else(|| anyhow::anyhow!("unresolved struct constructor type"))?;
                return self
                    .emit_struct_constructor(
                        &struct_definition,
                        result_type,
                        positional_arguments,
                        block,
                    )
                    .map(|(value, block)| (Some(value), block));
            }
            _ => anyhow::bail!(
                "callee '{}' does not resolve to a function",
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
                .expect("function call always produces at least one result");
            Ok((Some(result), current_block))
        }
    }

    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
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
                builder,
            );
            let index_value = builder.emit_sol_constant(index as i64, builder.types.ui64, &block);
            let field_address =
                builder.emit_sol_gep(struct_address, index_value, field_type, &block);

            let (argument_value, next_block) =
                self.expression_emitter.emit_value(&argument, block)?;
            block = next_block;
            let stored = TypeConversion::from_target_type(field_type, builder).emit(
                argument_value,
                builder,
                &block,
            );
            builder.emit_sol_store(stored, field_address, &block);
        }

        Ok((struct_address, block))
    }

    /// Emits a named-argument struct constructor `S({a: x, b: y, c: z})`.
    ///
    /// Matches each declared member by name and emits the values in
    /// declaration order. Members without a matching named argument cause
    /// the call to bail.
    fn emit_named_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        named_arguments: &slang_solidity_v2::ast::NamedArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
        for (index, member) in struct_definition.members().iter().enumerate() {
            let member_name = member.name().name();
            let argument = named_arguments
                .iter()
                .find(|argument| argument.name().name() == member_name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "named struct constructor missing field {member_name}",
                    )
                })?;
            let field_slang_type = member.get_type().expect("slang types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                builder,
            );
            let index_value = builder.emit_sol_constant(index as i64, builder.types.ui64, &block);
            let field_address =
                builder.emit_sol_gep(struct_address, index_value, field_type, &block);

            let value_expression = argument.value();
            let (argument_value, next_block) =
                self.expression_emitter.emit_value(&value_expression, block)?;
            block = next_block;
            let stored = TypeConversion::from_target_type(field_type, builder).emit(
                argument_value,
                builder,
                &block,
            );
            builder.emit_sol_store(stored, field_address, &block);
        }

        Ok((struct_address, block))
    }

    /// Emits a direct, named function call and returns all of its result
    /// values in declaration order.
    ///
    /// Unlike [`Self::emit_function_call`], this entry point does not handle
    /// explicit type conversions or built-in dispatch — it is intended for
    /// callers that need the full result tuple (e.g. tuple deconstruction).
    ///
    /// # Errors
    ///
    /// Returns an error if the call uses non-positional arguments, if the
    /// callee is not a named identifier, or if name resolution fails.
    pub fn emit_function_call_results(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        if let Some((values, block)) =
            self.try_emit_bare_call_results(call, positional_arguments, block)?
        {
            return Ok((values, block));
        }

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
    /// Returns the resolved MLIR name, the cast argument values, the
    /// declared return types, and the block in which the call should be
    /// emitted.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_built_in_member_access(access, None, block)?;
        Ok((
            value.expect("bare member access always produces a value"),
            block,
        ))
    }
}
