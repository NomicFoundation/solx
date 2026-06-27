//!
//! Solidity type-conversion call.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;

/// A one-argument Solidity type conversion.
pub struct TypeConversion {
    /// The original call expression.
    pub call: FunctionCallExpression,
    /// The converted expression.
    pub expression: Expression,
}

impl TypeConversion {
    /// Classifies a function call as a type conversion.
    pub fn from_call(call: &FunctionCallExpression) -> Option<Self> {
        if !call.is_type_conversion() {
            return None;
        }
        let ArgumentsDeclaration::PositionalArguments(positional) = call.arguments() else {
            return None;
        };
        if positional.len() != 1 {
            return None;
        }
        Some(Self {
            call: call.clone(),
            expression: positional.iter().next().expect("slang validated"),
        })
    }

    /// Emits the type conversion.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let target_type = AstType::resolve_optional(self.call.get_type(), &context.state.builder)
            .expect("slang validated");
        let BlockAnd { value, block } = self.expression.emit_as(target_type, context, block);
        BlockAnd {
            value: vec![value.into_mlir()],
            block,
        }
    }
}
