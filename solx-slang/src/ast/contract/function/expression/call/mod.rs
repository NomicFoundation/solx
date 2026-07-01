//!
//! Function call and member access expression emission.
//!

pub mod call_arguments;
pub mod call_kind;
pub mod identifier_builtin_call;
pub mod identifier_function_call;
pub mod member_builtin_call;
pub mod struct_construction;
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_values::EmitValues;

/// Lowers function call and member access expressions to MLIR.
pub struct CallContext<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    pub expression_context: &'emitter ExpressionContext<'state, 'context, 'block>,
}

impl<'context: 'block, 'block> EmitValues<'context, 'block> for FunctionCallExpression {
    /// Emits a function call, yielding its result values in declaration order: none for a void
    /// callee, one for a common callee, several for a tuple-returning call.
    fn emit_values<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let emitter = CallContext::new(context);
        let callee = self.operand();
        let ArgumentsDeclaration::PositionalArguments(arguments) = &self.arguments() else {
            unreachable!("only positional arguments supported");
        };
        match CallKind::from_call(self, &callee, arguments) {
            CallKind::StructConstruction(struct_definition) => {
                let result_type = context
                    .resolve_slang_type(self.get_type())
                    .expect("slang types every struct constructor");
                let (value, block) = emitter.emit_struct_constructor(
                    &struct_definition,
                    result_type,
                    arguments,
                    block,
                );
                BlockAnd {
                    block,
                    value: vec![value],
                }
            }
            CallKind::TypeConversion => {
                let (value, block) = emitter.emit_type_conversion(self, arguments, block);
                BlockAnd {
                    block,
                    value: vec![value],
                }
            }
            CallKind::IdentifierBuiltinCall => {
                let (value, block) = emitter.emit_built_in(self, &callee, arguments, block);
                BlockAnd {
                    block,
                    value: value.into_iter().collect(),
                }
            }
            CallKind::MemberBuiltinCall(access) => {
                let (value, block) =
                    emitter.emit_built_in_member_access(&access, Some(arguments), block);
                BlockAnd {
                    block,
                    value: value.into_iter().collect(),
                }
            }
            CallKind::IdentifierFunctionCall(function_definition) => {
                emitter.emit_function_call(&function_definition, arguments, block)
            }
        }
    }
}

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_context: &'emitter ExpressionContext<'state, 'context, 'block>) -> Self {
        Self { expression_context }
    }

    /// Emits an explicit one-argument type conversion (`uint256(x)`, `address(x)`).
    fn emit_type_conversion(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let first = arguments.iter().next().expect("type conversion has one argument");
        let target_type = self
            .expression_context
            .resolve_slang_type(call.get_type())
            .expect("slang types every type conversion target");
        let BlockAnd { value, block } =
            first.emit_as(target_type, self.expression_context, block);
        (value, block)
    }

    /// Dispatches a built-in reached by bare identifier or by member access whose result type comes
    /// from the call, returning its optional value.
    fn emit_built_in(
        &self,
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        if let Some(result) = self.try_emit_built_in_call(callee, arguments, block) {
            return result;
        }
        if let Some((value, block)) =
            self.try_emit_built_in_call_expression(call, arguments, block)
        {
            return (Some(value), block);
        }
        self.emit_built_in_member_access(
            match callee {
                Expression::MemberAccessExpression(access) => access,
                _ => unreachable!("identifier built-in was already dispatched"),
            },
            Some(arguments),
            block,
        )
    }

    /// Emits a bare member access expression (e.g. `tx.origin`, `msg.sender`).
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let (value, block) = self.emit_built_in_member_access(access, None, block);
        (
            value.expect("bare member access always produces a value"),
            block,
        )
    }
}
