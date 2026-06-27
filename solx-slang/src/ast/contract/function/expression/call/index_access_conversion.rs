//!
//! Array-type conversions represented as index-access callees.
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

/// A conversion call where the callee is an array type expression.
pub struct IndexAccessConversion {
    /// The full call expression.
    pub call: FunctionCallExpression,
    /// The expression being converted.
    pub argument: Expression,
}

impl IndexAccessConversion {
    /// Classifies an array-type conversion call.
    pub fn from_call(call: &FunctionCallExpression, callee: &Expression) -> Option<Self> {
        let Expression::IndexAccessExpression(array_type) = callee else {
            return None;
        };
        if array_type.start().is_some() || array_type.end().is_some() || array_type.is_slice() {
            return None;
        }
        let ArgumentsDeclaration::PositionalArguments(positional) = call.arguments() else {
            unreachable!("named arguments on an array-type cast are not supported");
        };
        let argument = positional.iter().next().expect("slang validated");
        Some(Self {
            call: call.clone(),
            argument,
        })
    }

    /// Emits the conversion.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let target_type = AstType::resolve_optional(self.call.get_type(), &context.state.builder)
            .expect("slang validated");
        let BlockAnd { value, block } = self.argument.emit_as(target_type, context, block);
        BlockAnd {
            value: vec![value.into_mlir()],
            block,
        }
    }
}
