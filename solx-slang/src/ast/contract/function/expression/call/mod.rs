//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use anyhow::Context as _;
use melior::ir::BlockLike;
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
use slang_solidity_v2::ast::Type as SlangType;
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

        // Named-argument calls to a directly-named function (`f({b: 2, a: 1})`)
        // reorder the arguments to the callee's declared parameter order and
        // dispatch as an ordinary positional call.
        if let ArgumentsDeclaration::NamedArguments(named_arguments) = &call.arguments()
            && let Expression::Identifier(callee_identifier) = call.operand()
            && let Some(Definition::Function(function_definition)) =
                callee_identifier.resolve_to_definition()
        {
            return self.emit_named_function_call(&function_definition, named_arguments, block);
        }

        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        let callee = call.operand();

        // A single-field struct constructor `S(x)` is reported as a "type
        // conversion" by slang's CST heuristic (the callee is a type name), but
        // it must build a struct via `emit_struct_constructor` — not cast the
        // argument to the struct type (which `sol.cast` rejects: it is
        // integer-only). Multi-field structs have >1 argument and already skip
        // this branch; route the single-field case to the `Definition::Struct`
        // dispatch below.
        //
        // Restrict this to a value-typed field: when the sole field is a
        // reference type (nested struct / array), `emit_struct_constructor`
        // emits a `sol.copy` the backend cannot yet lower (EVMUtil.cpp NYI for
        // copying aggregate members). Those keep the existing type-conversion
        // path to avoid a compile regression — they were already mis-compiled
        // (wrong value) rather than failing to compile.
        let callee_is_struct_constructor = match &callee {
            Expression::Identifier(identifier) => {
                match identifier.resolve_to_definition() {
                    Some(Definition::Struct(struct_definition)) => struct_definition
                        .members()
                        .iter()
                        .next()
                        .and_then(|member| member.get_type())
                        .is_some_and(|field_type| !field_type.is_reference_type()),
                    _ => false,
                }
            }
            _ => false,
        };

        if call.is_type_conversion()
            && positional_arguments.len() == 1
            && !callee_is_struct_constructor
        {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");

            // `E(x)` (integer -> enum): slang surfaces no call type for this
            // conversion, and it lowers to `sol.enum_cast` (not `sol.cast`).
            // Detect an enum callee, coerce the argument to `ui256`, and bridge.
            if let Expression::Identifier(callee_identifier) = &callee
                && let Some(Definition::Enum(enum_definition)) =
                    callee_identifier.resolve_to_definition()
            {
                let (value, block) = self.expression_emitter.emit_value(&first, block)?;
                let builder = &self.expression_emitter.state.builder;
                let member_count = enum_definition.members().iter().count();
                let max = u8::try_from(member_count.saturating_sub(1))
                    .expect("enum member count fits in u8");
                let enum_type = builder.types.enumeration(max.into());
                let raw = TypeConversion::from_target_type(builder.types.ui256, builder)
                    .emit(value, builder, &block);
                let result = block
                    .append_operation(
                        solx_mlir::ods::sol::EnumCastOperation::builder(
                            builder.context,
                            builder.unknown_location,
                        )
                        .inp(raw)
                        .out(enum_type)
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.enum_cast always produces one result")
                    .into();
                return Ok((Some(result), block));
            }

            let target_type = self
                .expression_emitter
                .resolve_slang_type(call.get_type())
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

            // `emit_value_for_target` materializes a string literal directly as a
            // `fixedbytes<N>` constant when the target is `bytesN` (e.g.
            // `bytes32("abc")`); `sol.bytes_cast` rejects a dynamic-string
            // operand, so a plain `emit_value` would fail here.
            let (value, block) = self
                .expression_emitter
                .emit_value_for_target(&first, target_type, block)?;
            let builder = &self.expression_emitter.state.builder;
            let result =
                TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
            return Ok((Some(result), block));
        }

        // `super.f(args)` — an internal call that skips the current contract's
        // own override and dispatches to the next implementation up the C3
        // linearisation. The redirect (built against the most-derived contract's
        // linearised bases) names the target node; this is correct even in a
        // diamond, where slang's lexical resolution of `access.member()` would
        // pick the wrong override. We emit a plain internal `sol.call` (no
        // receiver) to that node's registered symbol.
        if let Expression::MemberAccessExpression(access) = &callee
            && matches!(access.operand(), Expression::SuperKeyword(_))
        {
            let redirect_id = self
                .expression_emitter
                .state
                .super_redirect
                .get(&access.node_id())
                .copied();
            let setup = match redirect_id {
                Some(target_id) => {
                    self.emit_call_setup_by_id(target_id, positional_arguments, block)
                }
                // No redirect entry (e.g. a `super` call slang typed but the
                // linearised re-resolution did not reach): fall back to the
                // lexically-resolved target.
                None => {
                    let Some(Definition::Function(base_function)) =
                        access.member().resolve_to_definition()
                    else {
                        anyhow::bail!("super member access does not resolve to a function");
                    };
                    self.emit_call_setup(&base_function, positional_arguments, block)
                }
            };
            let (mlir_name, argument_values, return_types, current_block) =
                setup.context("resolving super call")?;
            if return_types.is_empty() {
                self.expression_emitter.state.builder.emit_sol_call(
                    mlir_name,
                    &argument_values,
                    &[],
                    &current_block,
                )?;
                return Ok((None, current_block));
            }
            let result = self
                .expression_emitter
                .state
                .builder
                .emit_sol_call(mlir_name, &argument_values, return_types, &current_block)?
                .expect("function call always produces at least one result");
            return Ok((Some(result), current_block));
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

        // Internal library function call — direct `L.f(args)` or a `using for`
        // `x.f(args)`. Must precede the member-access built-in dispatch below:
        // that path's `this.f(args)` branch would otherwise consume `x.f(args)`
        // without prepending the `x` receiver as the implicit `self`. Shared
        // with the multi-result path so a tuple-returning library function
        // (`return _s.reverse()`) yields all of its results.
        if let Some((results, block)) =
            self.try_emit_library_call(&callee, positional_arguments, block)?
        {
            return Ok((results.into_iter().next(), block));
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
                let mut call_value = None;
                for option in call_options.options().iter() {
                    let value_expression = option.value();
                    let (value, next) = self
                        .expression_emitter
                        .emit_value(&value_expression, current_block)?;
                    current_block = next;
                    // Capture `{value: v}` to forward as the external call's
                    // wei value; other options (gas, salt) are evaluated for
                    // side effects only.
                    if option.name().name() == "value" {
                        let builder = &self.expression_emitter.state.builder;
                        let cast = TypeConversion::from_target_type(builder.types.ui256, builder)
                            .emit(value, builder, &current_block);
                        call_value = Some(cast);
                    }
                }
                return self.emit_built_in_member_access_with_value(
                    access,
                    Some(positional_arguments),
                    call_value,
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
            // Non-identifier callee (e.g. `arr[i]()` array of function
            // pointers). If it has function-pointer type, call indirectly.
            if let Some(function_slang_type) = callee.get_type()
                && matches!(&function_slang_type, SlangType::Function(_))
            {
                return self.emit_indirect_call(
                    &callee,
                    &function_slang_type,
                    positional_arguments,
                    block,
                );
            }
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
            // Calling through an internal function pointer: the callee is a
            // variable/parameter/state-variable of function type. Load the
            // `func_ref` value and emit `sol.icall`.
            Some(
                Definition::Variable(_)
                | Definition::Parameter(_)
                | Definition::StateVariable(_),
            ) => {
                let function_slang_type = callee_identifier
                    .get_type()
                    .ok_or_else(|| anyhow::anyhow!("unresolved function-pointer type"))?;
                return self.emit_indirect_call(
                    &callee,
                    &function_slang_type,
                    positional_arguments,
                    block,
                );
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

    /// Emits an indirect call through an internal function pointer. The
    /// callee expression evaluates to a `func_ref` value (an identifier
    /// naming a function-typed variable, an array element `arr[i]`, etc.)
    /// which drives a `sol.icall`.
    fn emit_indirect_call(
        &self,
        callee: &Expression,
        function_slang_type: &SlangType,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // Load the function-pointer value.
        let (callee_value, mut current_block) =
            self.expression_emitter.emit_value(callee, block)?;

        // Derive parameter and result types from the pointer's function type.
        let SlangType::Function(function_type) = function_slang_type else {
            anyhow::bail!("indirect-call callee is not a function type");
        };
        let builder = &self.expression_emitter.state.builder;
        let parameter_types: Vec<Type<'context>> = function_type
            .parameter_types()
            .iter()
            .map(|t| TypeConversion::resolve_slang_type(t, None, builder))
            .collect();
        let result_types: Vec<Type<'context>> = match function_type.return_type() {
            SlangType::Void(_) => Vec::new(),
            SlangType::Tuple(tuple) => tuple
                .types()
                .iter()
                .map(|t| TypeConversion::resolve_slang_type(t, None, builder))
                .collect(),
            other => vec![TypeConversion::resolve_slang_type(&other, None, builder)],
        };

        // Evaluate and cast arguments to the declared parameter types.
        let mut argument_values = Vec::with_capacity(positional_arguments.len());
        for argument in positional_arguments.iter() {
            let (value, next) = self.expression_emitter.emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        let builder = &self.expression_emitter.state.builder;
        for (value, parameter_type) in argument_values.iter_mut().zip(parameter_types.iter()) {
            *value = TypeConversion::from_target_type(*parameter_type, builder)
                .emit(*value, builder, &current_block);
        }

        // External function pointers dispatch through a real CALL
        // (`sol.ext_icall`); internal ones through `sol.icall`.
        let results = if function_type.is_externally_visible() {
            let zero_value = builder.emit_sol_constant(0, builder.types.ui256, &current_block);
            builder.emit_sol_ext_icall(
                callee_value,
                &argument_values,
                &result_types,
                zero_value,
                &current_block,
            )?
        } else {
            builder.emit_sol_icall(callee_value, &argument_values, &result_types, &current_block)?
        };
        Ok((results.into_iter().next(), current_block))
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
            // Reference-typed fields (nested struct, array, mapping) take the
            // argument as a pointer and must be deep-copied via `sol.copy`.
            // Value-typed fields use a normal cast + store.
            if field_slang_type.is_reference_type() {
                builder.emit_sol_copy(argument_value, field_address, &block);
            } else {
                let stored = TypeConversion::from_target_type(field_type, builder).emit(
                    argument_value,
                    builder,
                    &block,
                );
                builder.emit_sol_store(stored, field_address, &block);
            }
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
            if field_slang_type.is_reference_type() {
                builder.emit_sol_copy(argument_value, field_address, &block);
            } else {
                let stored = TypeConversion::from_target_type(field_type, builder).emit(
                    argument_value,
                    builder,
                    &block,
                );
                builder.emit_sol_store(stored, field_address, &block);
            }
        }

        Ok((struct_address, block))
    }

    /// Emits a direct call written with named arguments (`f({b: 2, a: 1})`).
    ///
    /// Slang preserves named arguments in source order, so they are reordered
    /// to the callee's declared parameter order, emitted, coerced to the
    /// declared parameter types, and dispatched as an ordinary `sol.call`. The
    /// first result (if any) is returned, matching [`Self::emit_function_call`].
    ///
    /// # Errors
    ///
    /// Returns an error if a parameter is unnamed, an argument for a parameter
    /// is missing, or argument emission / signature resolution fails.
    fn emit_named_function_call(
        &self,
        function_definition: &FunctionDefinition,
        named_arguments: &slang_solidity_v2::ast::NamedArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut argument_values = Vec::new();
        let mut current_block = block;
        for parameter in function_definition.parameters().iter() {
            let parameter_name = parameter
                .name()
                .ok_or_else(|| anyhow::anyhow!("named call to a function with an unnamed parameter"))?
                .name();
            let argument = named_arguments
                .iter()
                .find(|argument| argument.name().name() == parameter_name)
                .ok_or_else(|| anyhow::anyhow!("named call missing argument `{parameter_name}`"))?;
            let (value, next_block) = self
                .expression_emitter
                .emit_value(&argument.value(), current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id())?;
        let builder = &self.expression_emitter.state.builder;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            *value = TypeConversion::from_target_type(param_type, builder).emit(
                *value,
                builder,
                &current_block,
            );
        }
        let results =
            builder.emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
        Ok((results.into_iter().next(), current_block))
    }

    /// Emits a `using for` / direct library call (`x.f(args)` or `L.f(args)`)
    /// when `callee` is a member access onto a pre-registered library internal
    /// function, returning every result value (so tuple-returning library
    /// functions work in both single- and multi-result contexts). For the
    /// `using for` form the receiver is prepended as the implicit `self`
    /// argument. Returns `None` when `callee` is not such a call, letting the
    /// caller fall through to ordinary dispatch.
    fn try_emit_library_call(
        &self,
        callee: &Expression,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::MemberAccessExpression(access) = callee else {
            return Ok(None);
        };
        let Some(Definition::Function(library_function)) = access.member().resolve_to_definition()
        else {
            return Ok(None);
        };
        if !self
            .expression_emitter
            .state
            .library_function_ids
            .contains(&library_function.node_id())
        {
            return Ok(None);
        }

        // A `using for` receiver (`x.f(args)`) is a value and becomes the
        // implicit `self`. A namespace qualifier — a library (`L.f(args)`) or an
        // import alias (`import "a.sol" as M; M.f(args)`) — is not a value, so it
        // contributes no `self` and only the explicit arguments are passed.
        let receiver_is_qualifier = matches!(
            access.operand(),
            Expression::Identifier(identifier)
                if matches!(
                    identifier.resolve_to_definition(),
                    Some(
                        Definition::Library(_)
                            | Definition::Import(_)
                            | Definition::ImportedSymbol(_)
                    )
                )
        );
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(library_function.node_id())?;

        let mut argument_values = Vec::new();
        let mut current_block = block;
        if !receiver_is_qualifier {
            let (self_value, next) = self
                .expression_emitter
                .emit_value(&access.operand(), current_block)?;
            argument_values.push(self_value);
            current_block = next;
        }
        for argument in positional_arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }

        let builder = &self.expression_emitter.state.builder;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            *value = TypeConversion::from_target_type(parameter_type, builder).emit(
                *value,
                builder,
                &current_block,
            );
        }
        let results = builder.emit_sol_call_results(
            mlir_name,
            &argument_values,
            return_types,
            &current_block,
        )?;
        Ok(Some((results, current_block)))
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

        // Multi-value `abi.decode(payload, (A, B, …))` in a tuple assignment
        // `(a, b) = abi.decode(...)` returns one value per requested type.
        if let Expression::MemberAccessExpression(access) = call.operand()
            && matches!(
                access.member().resolve_to_built_in(),
                Some(slang_solidity_v2::ast::BuiltIn::AbiDecode)
            )
        {
            return self.emit_abi_decode(call, positional_arguments, block);
        }

        if let Some((results, block)) =
            self.try_emit_library_call(&call.operand(), positional_arguments, block)?
        {
            return Ok((results, block));
        }

        // `(a, b) = recv.f(args)` / `this.f(args)` — a genuine external contract
        // call returning a tuple. Bare calls, `abi.decode`, and library calls
        // are already handled above, so any remaining member-access callee that
        // resolves to a function is an external call.
        if let Some((results, block)) =
            self.try_emit_external_call_results(call, positional_arguments, block)?
        {
            return Ok((results, block));
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
        // Virtual dispatch: a call resolving (lexically) to an overridden base
        // function is routed to the most-derived override. The redirect only
        // contains shadowed-override nodes, so non-virtual callees are
        // unaffected. `super` deliberately bypasses this (it calls
        // `emit_call_setup_by_id` directly with the linearised target).
        let node_id = function_definition.node_id();
        let call_id = self
            .expression_emitter
            .state
            .virtual_redirect
            .get(&node_id)
            .copied()
            .unwrap_or(node_id);
        self.emit_call_setup_by_id(call_id, positional_arguments, block)
    }

    /// Like [`Self::emit_call_setup`] but resolves the callee by its AST node
    /// id directly. Used for `super` dispatch, where the redirect names the
    /// target node (the lexically-resolved member would be the wrong override
    /// in a diamond).
    fn emit_call_setup_by_id<'a>(
        &'a self,
        definition_id: slang_solidity_v2::ast::NodeId,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [melior::ir::Type<'context>],
        BlockRef<'context, 'block>,
    )> {
        // Resolve the signature first so each argument can be emitted toward
        // its declared parameter type — notably so a string literal in a
        // `bytesN` parameter becomes a fixedbytes constant rather than a memory
        // string (see `emit_value_for_target`).
        let (mlir_name, parameter_types, return_types) =
            self.expression_emitter.state.resolve_function(definition_id)?;

        let mut argument_values = Vec::new();
        let mut current_block = block;
        for (index, argument) in positional_arguments.iter().enumerate() {
            let (value, next_block) = match parameter_types.get(index) {
                Some(&param_type) => self.expression_emitter.emit_value_for_target(
                    &argument,
                    param_type,
                    current_block,
                )?,
                None => self.expression_emitter.emit_value(&argument, current_block)?,
            };
            argument_values.push(value);
            current_block = next_block;
        }

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
