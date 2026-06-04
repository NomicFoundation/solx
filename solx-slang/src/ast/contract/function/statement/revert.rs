//!
//! Revert statement lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::RevertStatement;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::contract::function::statement::named_arguments::order_named_arguments;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers `revert ErrorName(args);` to a `sol.revert` carrying the error's
    /// canonical signature and ABI-encoded arguments, each coerced to its
    /// declared parameter type.
    ///
    /// `sol.revert` is not a dialect terminator, so lowering continues in the
    /// same block; the function epilogue supplies the structural terminator.
    pub fn emit_revert(
        &self,
        revert: &RevertStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(Definition::Error(error)) = revert.error().resolve_to_definition() else {
            unreachable!("a `revert Error(...)` target resolves to an error definition");
        };
        let parameters = error.parameters();
        let ordered_arguments = match revert.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                positional.iter().collect::<Vec<_>>()
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                order_named_arguments(&named, &parameters)
            }
        };

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let mut values: Vec<Value<'context, 'block>> = Vec::with_capacity(ordered_arguments.len());
        let mut block = block;
        for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
            let (value, next_block) = emitter.emit_value(&argument, block)?;
            block = next_block;
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("the binder types every error parameter"),
                None,
                &self.state.builder,
            );
            let value = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                value,
                &self.state.builder,
                &block,
            );
            values.push(value);
        }

        let signature = error
            .compute_canonical_signature()
            .expect("the binder computes a canonical signature for an error");
        self.state
            .builder
            .emit_sol_revert(&signature, &values, true, &block);
        Ok(Some(block))
    }

    /// Lowers the call form `revert()` / `revert("message")`.
    ///
    /// A non-empty string literal bakes the message into the op as the
    /// `Error(string)` payload; a runtime message expression (or an empty
    /// literal) is evaluated, coerced to `string memory`, and ABI-encoded under
    /// the `Error(string)` selector, exactly like `require(cond, expr)`.
    pub fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = call.arguments() else {
            unimplemented!("revert with named arguments");
        };
        let message = arguments.iter().next();
        match message {
            None => {
                self.state.builder.emit_sol_revert("", &[], false, &block);
                Ok(Some(block))
            }
            Some(Expression::StringExpression(string)) if !string.value().is_empty() => {
                let message = String::from_utf8(string.value())
                    .map_err(|_| anyhow::anyhow!("revert message is not valid UTF-8"))?;
                self.state
                    .builder
                    .emit_sol_revert(&message, &[], false, &block);
                Ok(Some(block))
            }
            Some(expression) => {
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (message, block) = emitter.emit_value(&expression, block)?;
                let builder = &self.state.builder;
                let string_memory = builder.types.string(DataLocation::Memory);
                let message = TypeConversion::from_target_type(string_memory, builder)
                    .emit(message, builder, &block);
                builder.emit_sol_revert("Error(string)", &[message], true, &block);
                Ok(Some(block))
            }
        }
    }
}
