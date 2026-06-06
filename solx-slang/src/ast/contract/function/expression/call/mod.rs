//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod call_kind;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use solx_utils::DataLocation;

use self::call_kind::CallKind;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

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
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("only positional arguments supported");
        };

        match self.classify_call(call, positional_arguments) {
            CallKind::TypeConversion => {
                let first = positional_arguments
                    .iter()
                    .next()
                    .expect("a type conversion has exactly one argument");
                let (value, block) = self.expression_emitter.emit_value(&first, block)?;
                let builder = &self.expression_emitter.state.builder;
                let target_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("slang types a type-conversion call");
                let result = TypeConversion::from_target_type(target_type, builder)
                    .emit(value, builder, &block);
                Ok((Some(result), block))
            }
            CallKind::BuiltInIdentifier(built_in) => {
                self.emit_built_in_call(built_in, positional_arguments, block)
            }
            CallKind::AbiDecode => {
                let (value, block) = self.emit_abi_decode(call, positional_arguments, block)?;
                Ok((Some(value), block))
            }
            CallKind::BuiltInMemberAccess(access) => {
                self.emit_built_in_member_access(&access, Some(positional_arguments), block)
            }
            CallKind::LocalFunction(function_definition) => {
                let callee_name = function_definition
                    .name()
                    .map(|identifier| identifier.name())
                    .unwrap_or_default();
                let (mlir_name, argument_values, return_types, current_block) = self
                    .emit_call_setup(&function_definition, positional_arguments, block)
                    .with_context(|| format!("resolving callee '{callee_name}'"))?;

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
            CallKind::StructConstructor(struct_definition) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("slang types a struct constructor call");
                self.emit_struct_constructor(
                    &struct_definition,
                    result_type,
                    positional_arguments,
                    block,
                )
                .map(|(value, block)| (Some(value), block))
            }
        }
    }

    /// Classifies a call expression into its [`CallKind`] ahead of emission,
    /// from positive, mutually-exclusive resolution facts rather than a
    /// speculative chain of fallible attempts.
    ///
    /// The arms are ordered to preserve the original dispatch precedence:
    /// type conversion, then identifier built-in, then the `abi.decode`
    /// member built-in, then other member built-ins, then a user-defined
    /// function or struct constructor. An unsupported callee shape is a loud
    /// `unimplemented!`.
    fn classify_call(
        &self,
        call: &FunctionCallExpression,
        positional_arguments: &PositionalArguments,
    ) -> CallKind {
        let callee = call.operand();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            return CallKind::TypeConversion;
        }

        if let Expression::Identifier(identifier) = &callee
            && let Some(built_in) = identifier.resolve_to_built_in()
            && Self::is_emittable_identifier_built_in(built_in, positional_arguments.len())
        {
            return CallKind::BuiltInIdentifier(built_in);
        }

        if let Expression::MemberAccessExpression(access) = &callee {
            if matches!(
                access.member().resolve_to_built_in(),
                Some(BuiltIn::AbiDecode)
            ) {
                return CallKind::AbiDecode;
            }
            return CallKind::BuiltInMemberAccess(access.clone());
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            unimplemented!("unsupported callee expression");
        };
        match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => {
                CallKind::LocalFunction(function_definition)
            }
            Some(Definition::Struct(struct_definition)) => {
                CallKind::StructConstructor(struct_definition)
            }
            _ => unimplemented!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            ),
        }
    }

    /// Returns whether an identifier-callee built-in is lowered directly with
    /// the given argument count. Built-ins not listed here (or with a
    /// mismatched arity) fall through to user-defined function resolution,
    /// preserving the original dispatch behavior.
    fn is_emittable_identifier_built_in(built_in: BuiltIn, argument_count: usize) -> bool {
        matches!(
            (built_in, argument_count),
            (BuiltIn::Assert, 1)
                | (BuiltIn::Require, 1 | 2)
                | (BuiltIn::Gasleft, 0)
                | (BuiltIn::Keccak256, 1)
                | (BuiltIn::Sha256, 1)
                | (BuiltIn::Ripemd160, 1)
                | (BuiltIn::Ecrecover, 4)
                | (BuiltIn::Addmod, 3)
                | (BuiltIn::Mulmod, 3)
        )
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
            unimplemented!("only positional arguments supported");
        };

        let Expression::Identifier(callee_identifier) = call.operand() else {
            unimplemented!("multi-result calls only support direct named function callees");
        };
        let Some(Definition::Function(function_definition)) =
            callee_identifier.resolve_to_definition()
        else {
            unimplemented!(
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

    /// Evaluates `arguments` left-to-right (via
    /// [`CallEmitter::emit_argument_values`]) and coerces each resulting value to
    /// its declared parameter type, returning the materialised argument values
    /// and the continuation block.
    ///
    /// The single argument eval-and-coerce primitive: every call site (internal,
    /// external, library, struct-constructor) delegates here rather than
    /// re-implementing the evaluation and zip-coerce loops. `pub` so the call
    /// fills in sibling modules reuse it.
    pub fn emit_coerced_arguments(
        &self,
        arguments: &PositionalArguments,
        parameter_types: &[melior::ir::Type<'context>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (mut argument_values, block) = self.emit_argument_values(arguments, block)?;
        let builder = &self.expression_emitter.state.builder;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(parameter_type, builder);
            *value = conversion.emit(*value, builder, &block);
        }
        Ok((argument_values, block))
    }

    /// Resolves the callee's MLIR signature, then evaluates and coerces the
    /// arguments to its declared parameter types.
    ///
    /// Returns the resolved MLIR name, the coerced argument values, the
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
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id())?;

        let (argument_values, current_block) =
            self.emit_coerced_arguments(positional_arguments, parameter_types, block)?;

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
