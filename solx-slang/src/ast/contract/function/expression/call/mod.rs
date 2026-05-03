//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionCallExpression;
use slang_solidity::backend::ir::ast::MemberAccessExpression;
use slang_solidity::backend::ir::ast::PositionalArguments;

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
    /// Resolves the callee by name and argument count, handling type
    /// conversions, built-in functions, and user-defined calls.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is unsupported, arguments contain
    /// unsupported constructs, or the function is undefined.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        // Resolve callee name for Identifier/PayableKeyword callees; None for
        // ElementaryType and MemberAccessExpression (handled as identity casts).
        let callee = call.operand();
        let callee_name: Option<String> = match &callee {
            Expression::Identifier(identifier) => Some(identifier.name()),
            Expression::PayableKeyword => Some("payable".to_owned()),
            _ => None,
        };

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
            let (value, block) = self.expression_emitter.emit_value(&first, block)?;
            let builder = &self.expression_emitter.state.builder;

            let target_type = self
                .expression_emitter
                .resolve_expression_type(call.node_id())
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

            let result =
                TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
            return Ok((Some(result), block));
        }

        // Non-conversion path: require named callee.
        let callee_name =
            callee_name.ok_or_else(|| anyhow::anyhow!("unsupported callee expression"))?;

        if let Some(block) = self.try_emit_built_in_call(&callee, positional_arguments, block)? {
            return Ok((None, block));
        }

        let (mlir_name, argument_values, return_types, current_block) =
            self.emit_call_setup(&callee_name, positional_arguments, block)?;

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

        let callee_name = match call.operand() {
            Expression::Identifier(identifier) => identifier.name(),
            _ => anyhow::bail!(
                "multi-result calls only support direct named function callees"
            ),
        };

        let (mlir_name, argument_values, return_types, current_block) =
            self.emit_call_setup(&callee_name, positional_arguments, block)?;

        let results = self.expression_emitter.state.builder.emit_sol_call_results(
            mlir_name,
            &argument_values,
            return_types,
            &current_block,
        )?;
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
        callee_name: &str,
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

        let argument_types: Vec<melior::ir::Type<'context>> =
            argument_values.iter().map(|value| value.r#type()).collect();
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(callee_name, &argument_types)?;

        let builder = &self.expression_emitter.state.builder;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, builder);
            *value = conversion.emit(*value, builder, &current_block);
        }

        Ok((mlir_name, argument_values, return_types, current_block))
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// rather than string-matching the member name.
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_built_in_member_access(access, block)
    }
}
