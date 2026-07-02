//!
//! Function call and member access expression emission.
//!

pub mod call_arguments;
pub mod call_kind;
pub mod contract_creation;
pub mod external_member_call;
pub mod function_pointer_call;
pub mod identifier_builtin_call;
pub mod identifier_function_call;
pub mod member_builtin_call;
pub mod new_expression_call;
pub mod struct_construction;
pub mod try_call;
pub mod try_call_kind;
pub mod try_new_expression;
pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_kind::CallKind;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::call_options::CallOptions;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
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
        let arguments = self.arguments();
        let (call_value, salt, call_gas, block, callee) = match self.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => {
                let (value, salt, gas, block) = CallOptions(&options).capture(context, block);
                (
                    value,
                    salt,
                    gas,
                    block,
                    options.operand().unwrap_parentheses(),
                )
            }
            other => (None, None, None, block, other),
        };
        match CallKind::from_call(self, &callee, &arguments) {
            CallKind::StructConstruction(struct_definition) => {
                let result_type = context
                    .resolve_slang_type(self.get_type())
                    .expect("slang types every struct constructor");
                let (value, block) = emitter.emit_struct_constructor(
                    &struct_definition,
                    result_type,
                    &arguments,
                    block,
                );
                BlockAnd {
                    block,
                    value: vec![value],
                }
            }
            CallKind::TypeConversion => {
                let (value, block) =
                    emitter.emit_type_conversion(self, &emitter.positional(&arguments), block);
                BlockAnd {
                    block,
                    value: vec![value],
                }
            }
            CallKind::FunctionPointerCall(callee) => {
                emitter.emit_function_pointer_call(&callee, &emitter.positional(&arguments), block)
            }
            CallKind::IdentifierBuiltinCall => {
                let (value, block) =
                    emitter.emit_built_in(self, &callee, &emitter.positional(&arguments), block);
                BlockAnd { block, value }
            }
            CallKind::ExternalMemberCall(access, function_definition) => emitter
                .emit_external_member_call(
                    &access,
                    &function_definition,
                    &arguments,
                    call_value,
                    call_gas,
                    block,
                ),
            CallKind::MemberBuiltinCall(access) => {
                if let Some(
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall),
                ) = access.member().resolve_to_built_in()
                {
                    let (value, block) = emitter.emit_bare_call(
                        &access,
                        kind,
                        &emitter.positional(&arguments),
                        call_value,
                        call_gas,
                        block,
                    );
                    return BlockAnd { block, value };
                }
                let (value, block) = emitter.emit_built_in_member_access(
                    &access,
                    Some(&emitter.positional(&arguments)),
                    block,
                );
                BlockAnd {
                    block,
                    value: value.into_iter().collect(),
                }
            }
            CallKind::NewExpressionCall => {
                let (value, block) = emitter.emit_new_expression(
                    self,
                    &emitter.positional(&arguments),
                    call_value,
                    salt,
                    block,
                );
                BlockAnd {
                    block,
                    value: vec![value],
                }
            }
            CallKind::IdentifierFunctionCall(function_definition) => {
                emitter.emit_function_call(&function_definition, &arguments, block)
            }
        }
    }
}

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_context: &'emitter ExpressionContext<'state, 'context, 'block>) -> Self {
        Self { expression_context }
    }

    /// Unwraps the positional argument list of a call that does not accept named arguments.
    fn positional(&self, arguments: &ArgumentsDeclaration) -> PositionalArguments {
        let ArgumentsDeclaration::PositionalArguments(positional) = arguments else {
            unreachable!("only direct calls and struct constructors accept named arguments");
        };
        positional.clone()
    }

    /// Emits an explicit one-argument type conversion (`uint256(x)`, `bytes4(x)`, `E(x)`).
    ///
    /// A string literal toward a `byte` / `bytesN` target folds to a fixed-bytes constant through
    /// [`EmitAs`]. Every other conversion classifies from its Slang source and target types, so that
    /// the reverse `uintN(bytesN)` and `uint8(E)` directions route through `sol.bytes_cast` and
    /// `sol.enum_cast` rather than the integer `sol.cast`.
    fn emit_type_conversion(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let first = arguments.iter().next().expect("type conversion has one argument");
        let target_slang = call.get_type().expect("slang types every type conversion target");
        let target_type = self
            .expression_context
            .resolve_slang_type(Some(target_slang.clone()))
            .expect("slang types every type conversion target");
        if let Expression::StringExpression(_) = first {
            let BlockAnd { value, block } =
                first.emit_as(target_type, self.expression_context, block);
            return (value, block);
        }
        let source_slang = first.get_type().expect("slang types every conversion operand");
        let BlockAnd { value, block } = first.emit(self.expression_context, block);
        let value = TypeConversion::from_slang_conversion(
            &source_slang,
            &target_slang,
            target_type,
            self.expression_context.state,
        )
        .emit(value, self.expression_context.state, &block);
        (value, block)
    }

    /// Dispatches a built-in reached by bare identifier or by member access whose result type comes
    /// from the call, returning its result values in declaration order: none for a statement-style
    /// built-in, one for a value-producing built-in, several for a multi-value `abi.decode`.
    fn emit_built_in(
        &self,
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        if let Some((value, block)) = self.try_emit_built_in_call(callee, arguments, block) {
            return (value.into_iter().collect(), block);
        }
        if let Some((values, block)) =
            self.try_emit_built_in_call_expression(call, arguments, block)
        {
            return (values, block);
        }
        let (value, block) = self.emit_built_in_member_access(
            match callee {
                Expression::MemberAccessExpression(access) => access,
                _ => unreachable!("identifier built-in was already dispatched"),
            },
            Some(arguments),
            block,
        );
        (value.into_iter().collect(), block)
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
