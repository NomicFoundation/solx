//!
//! Function and built-in call lowering.
//!

/// Built-in function call lowering.
pub mod built_in;
/// Struct constructor call lowering.
pub mod struct_constructor;
/// Type conversions between Solidity and Sol dialect MLIR types.
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::PositionalArguments;

use self::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::ExpressionEmitter;

/// Lowers a `FunctionCallExpression`, classifying the callee and routing to the
/// matching call kind.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter, for recursive subexpression lowering.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a call emitter borrowing its parent expression emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Lowers a function call by trying each call-kind handler in turn.
    ///
    /// External / library calls, `new`, and named-argument calls are lowered by
    /// later domains.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = &call.arguments() else {
            unimplemented!("named-argument call lowering");
        };

        if let Some(result) = self.try_emit_type_conversion(call, arguments, block)? {
            return Ok(result);
        }

        let callee = call.operand();
        if let Some(result) = self.try_emit_built_in_call(&callee, arguments, block)? {
            return Ok(result);
        }

        if let Some(result) = self.try_emit_member_built_in_call(call, arguments, block)? {
            return Ok(result);
        }

        // A bare call discarded in statement position (`addr.call(data);`) keeps
        // only the success status; the return data is dropped.
        if let Some((values, block)) = self.try_emit_bare_call_results(call, arguments, block)? {
            return Ok((values.into_iter().next(), block));
        }

        if let Some(result) = self.try_emit_struct_constructor(call, arguments, block)? {
            return Ok(result);
        }

        // `new T[](n)` / `new bytes(n)` — a zeroed dynamic memory allocation.
        if let Some(result) = self.try_emit_new_array(call, arguments, block)? {
            return Ok(result);
        }

        // An external member call (`recv.f(args)` / `this.f(args)`) used in
        // value position keeps its first declared result.
        if let Some((values, block)) =
            self.try_emit_external_call_results(call, arguments, block)?
        {
            return Ok((values.into_iter().next(), block));
        }

        self.emit_internal_call(&callee, arguments, block)
    }

    /// Emits a direct internal call `f(args)` to a contract function as a
    /// `sol.call` to its registered symbol.
    ///
    /// Calls through function-pointer values and member-access callees
    /// (`c.g(...)`, `L.f(...)`) defer to later domains.
    fn emit_internal_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(identifier) = callee else {
            unimplemented!("call lowering: {:?}", std::mem::discriminant(callee));
        };
        let Some(Definition::Function(function)) = identifier.resolve_to_definition() else {
            unimplemented!("call to non-function callee: {}", identifier.name());
        };

        let (callee_name, argument_values, return_types, block) =
            self.emit_call_setup(&function, arguments, block)?;
        let result = self.expression_emitter.state.builder.emit_sol_call(
            callee_name,
            &argument_values,
            return_types,
            &block,
        )?;
        Ok((result, block))
    }

    /// Emits a direct internal call `f(args)` whose results are bound
    /// individually (tuple deconstruction `(a, b) = f();`), returning one value
    /// per declared return.
    ///
    /// Only direct internal function callees are supported; member-access,
    /// library, and function-pointer callees defer to later domains.
    pub fn emit_function_call_results(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = &call.arguments() else {
            unimplemented!("named-argument call lowering");
        };

        // `(bool ok, bytes memory d) = addr.call(payload)` — a bare low-level
        // call yielding both the success status and the raw return data.
        if let Some(result) = self.try_emit_bare_call_results(call, arguments, block)? {
            return Ok(result);
        }

        // `(a, b) = recv.f(args)` — an external member call returning a tuple.
        if let Some(result) = self.try_emit_external_call_results(call, arguments, block)? {
            return Ok(result);
        }

        let Expression::Identifier(identifier) = call.operand() else {
            unimplemented!(
                "multi-result call callee: {:?}",
                std::mem::discriminant(&call.operand())
            );
        };
        let Some(Definition::Function(function)) = identifier.resolve_to_definition() else {
            unimplemented!(
                "multi-result call to non-function callee: {}",
                identifier.name()
            );
        };

        let (callee_name, argument_values, return_types, block) =
            self.emit_call_setup(&function, arguments, block)?;
        let results = self
            .expression_emitter
            .state
            .builder
            .emit_sol_call_results(callee_name, &argument_values, return_types, &block)?;
        Ok((results, block))
    }

    /// Resolves a callee's registered signature and emits its arguments,
    /// each coerced to its declared parameter type.
    ///
    /// Returns the callee's MLIR symbol, the argument values, its declared
    /// return types, and the continuation block.
    fn emit_call_setup<'a>(
        &'a self,
        function: &FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [Type<'context>],
        BlockRef<'context, 'block>,
    )> {
        let (callee_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function.node_id())?;

        let mut argument_values = Vec::with_capacity(arguments.len());
        let mut block = block;
        for (index, argument) in arguments.iter().enumerate() {
            let (value, next_block) = self.expression_emitter.emit_value(&argument, block)?;
            block = next_block;
            let value = match parameter_types.get(index) {
                Some(&parameter_type) => {
                    let builder = &self.expression_emitter.state.builder;
                    TypeConversion::from_target_type(parameter_type, builder)
                        .emit(value, builder, &block)
                }
                None => value,
            };
            argument_values.push(value);
        }
        Ok((callee_name, argument_values, return_types, block))
    }

    /// Emits an explicit type conversion `T(x)` (e.g. `uint256(x)`, `uint8(x)`)
    /// as a `sol.cast`/`sol.address_cast`/comparison via [`TypeConversion`].
    ///
    /// Returns `None` when the call is not a single-argument type conversion.
    /// Conversions slang leaves untyped (enum, `bytes` of a constant) and
    /// single-field struct constructors (which slang also reports as
    /// conversions) defer to later domains.
    fn try_emit_type_conversion(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if !call.is_type_conversion() || arguments.len() != 1 {
            return Ok(None);
        }
        let argument = arguments
            .iter()
            .next()
            .expect("argument count checked to be one");

        // `E(x)` (integer -> enum): slang reports a type conversion but surfaces
        // no call type, and it lowers to `sol.enum_cast` rather than `sol.cast`.
        if let Some(result) = self.try_emit_enum_conversion(&call.operand(), &argument, block)? {
            return Ok(Some(result));
        }

        let Some(target_slang_type) = call.get_type() else {
            unimplemented!("untyped type conversion");
        };
        let builder = &self.expression_emitter.state.builder;
        let target_type = TypeConversion::resolve_slang_type(&target_slang_type, None, builder);
        let (value, block) = self.expression_emitter.emit_value(&argument, block)?;
        let result =
            TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
        Ok(Some((Some(result), block)))
    }

    /// Tries to lower an enum conversion `E(x)` (integer → enum) to a
    /// `sol.enum_cast`. Slang reports such a conversion as a type conversion but
    /// gives it no call type, so it cannot take the ordinary `sol.cast` path. The
    /// callee may be a bare `E` or a qualified `L.E` / `C.E` (a library- or
    /// contract-nested enum). Returns `Ok(None)` when the callee is not an enum,
    /// so the caller falls through to the ordinary conversion path.
    fn try_emit_enum_conversion(
        &self,
        callee: &Expression,
        argument: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let target = match callee {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => return Ok(None),
        };
        let Some(Definition::Enum(enum_definition)) = target else {
            return Ok(None);
        };
        let (value, block) = self.expression_emitter.emit_value(argument, block)?;
        let builder = &self.expression_emitter.state.builder;
        let member_count = enum_definition.members().iter().count();
        let enum_type = builder.types.enumeration_for_member_count(member_count);
        let raw = TypeConversion::from_target_type(builder.types.ui256, builder)
            .emit(value, builder, &block);
        let result = builder.emit_sol_enum_cast(raw, enum_type, &block);
        Ok(Some((Some(result), block)))
    }

    /// Coerces each evaluated argument to its declared parameter type in place
    /// (a narrow literal to its parameter width, an address/contract to the
    /// declared interface, …). Extra arguments past the known parameter types
    /// are left unchanged.
    fn coerce_arguments(
        &self,
        argument_values: &mut [Value<'context, 'block>],
        parameter_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) {
        let builder = &self.expression_emitter.state.builder;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            *value = TypeConversion::from_target_type(parameter_type, builder)
                .emit(*value, builder, block);
        }
    }

    /// Evaluates a `{value: v, gas: g, ...}` call-options layer for its side
    /// effects and returns the `value` option (coerced to `ui256`) when
    /// present. The `gas` option is left to the backend's default stipend.
    fn capture_call_value(
        &self,
        call_options: &CallOptionsExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut block = block;
        let mut call_value = None;
        for option in call_options.options().iter() {
            let (value, next_block) = self.expression_emitter.emit_value(&option.value(), block)?;
            block = next_block;
            if option.name().name() == "value" {
                let builder = &self.expression_emitter.state.builder;
                call_value = Some(
                    TypeConversion::from_target_type(builder.types.ui256, builder)
                        .emit(value, builder, &block),
                );
            }
        }
        Ok((call_value, block))
    }
}
