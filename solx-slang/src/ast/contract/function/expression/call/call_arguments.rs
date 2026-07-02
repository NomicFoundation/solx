//!
//! Solidity call arguments after named-argument ordering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Function;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

/// Solidity arguments ordered against the declaration they call.
pub struct CallArguments {
    /// Argument expressions in declaration order.
    pub expressions: Vec<Expression>,
}

impl CallArguments {
    /// Builds arguments already known to be in call order.
    pub fn ordered(expressions: Vec<Expression>) -> Self {
        Self { expressions }
    }

    /// Builds arguments ordered against declaration parameter IDs.
    pub fn for_parameter_ids(arguments: &ArgumentsDeclaration, parameter_ids: &[NodeId]) -> Self {
        Self {
            expressions: arguments
                .ordered_by(parameter_ids)
                .expect("slang validated"),
        }
    }

    /// Builds member-call arguments ordered after the implicit receiver parameter.
    pub fn after_receiver(arguments: &ArgumentsDeclaration, parameter_ids: &[NodeId]) -> Self {
        Self::for_parameter_ids(arguments, &parameter_ids[1..])
    }

    /// Builds arguments from a positional-only call.
    pub fn positional(arguments: &ArgumentsDeclaration) -> Self {
        let ArgumentsDeclaration::PositionalArguments(positional) = arguments else {
            unreachable!("slang validated");
        };
        Self {
            expressions: positional.iter().collect(),
        }
    }

    /// Emits each argument left-to-right into its raw value.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let mut values = Vec::with_capacity(self.expressions.len());
        let mut block = block;
        for argument in &self.expressions {
            let BlockAnd { value, block: next } = argument.emit(context, block);
            values.push(value.into_mlir());
            block = next;
        }
        BlockAnd {
            value: values,
            block,
        }
    }

    /// Emits each argument left-to-right, coercing to the declared parameter type.
    pub fn emit_as<'state, 'context: 'block, 'block>(
        &self,
        parameter_types: &[Type<'context>],
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        self.expressions
            .as_slice()
            .emit_as(parameter_types, context, block)
    }

    /// Emits each argument coerced to `function`'s parameters, then calls it, yielding its results.
    pub fn emit_call<'state, 'context: 'block, 'block>(
        &self,
        function: &Function<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let BlockAnd {
            value: argument_values,
            block,
        } = self.emit_as(&function.parameter_types, context, block);
        let results = function.call(&argument_values, context.state, &block);
        BlockAnd {
            value: results,
            block,
        }
    }
}
