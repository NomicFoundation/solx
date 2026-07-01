//!
//! Solidity type-conversion call: an elementary or user-defined cast, and the array-type cast
//! written with an index-access callee (`uint8[](value)`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;

use solx_mlir::Type as AstType;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;

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

    /// Classifies an array-type cast written with an index-access callee, `uint8[](value)`.
    pub fn from_index_access(call: &FunctionCallExpression, callee: &Expression) -> Option<Self> {
        let Expression::IndexAccessExpression(array_type) = callee else {
            return None;
        };
        if array_type.start().is_some() || array_type.end().is_some() || array_type.is_slice() {
            return None;
        }
        let ArgumentsDeclaration::PositionalArguments(positional) = call.arguments() else {
            unreachable!("named arguments on an array-type cast are not supported");
        };
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
        let target_type = AstType::resolve_optional(self.call.get_type(), context.state)
            .expect("slang validated");
        let BlockAnd { value, block } = self.expression.emit_as(target_type, context, block);
        BlockAnd {
            value: vec![value.into_mlir()],
            block,
        }
    }
}
