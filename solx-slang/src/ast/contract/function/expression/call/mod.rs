//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod type_conversion;

use melior::ir::BlockRef;
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

use solx_mlir::Function;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_utils::DataLocation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_values::EmitValues;

use self::type_conversion::TypeConversion;

/// The one emission kind a function call's callee resolves to. The variants are mutually exclusive
/// and tested in declaration order, so an earlier match wins.
enum CallKind {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A built-in invoked by bare identifier (`require`, `keccak256`) or a built-in reached through
    /// member access whose result type comes from the call (`abi.decode`).
    IdentifierBuiltinCall,
    /// A built-in reached through member access (`address.balance`, `abi.encode`).
    MemberBuiltinCall(MemberAccessExpression),
    /// A direct call to a named function.
    IdentifierFunctionCall(FunctionDefinition),
}

impl CallKind {
    /// Classifies `call`'s callee into the single kind that emits it.
    fn from_call(
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &PositionalArguments,
    ) -> Self {
        if let Expression::Identifier(identifier) = callee
            && let Some(Definition::Struct(struct_definition)) = identifier.resolve_to_definition()
        {
            return Self::StructConstruction(struct_definition);
        }
        if call.is_type_conversion() && arguments.len() == 1 {
            return Self::TypeConversion;
        }
        if let Expression::Identifier(identifier) = callee
            && identifier.resolve_to_built_in().is_some()
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee
            && matches!(access.member().resolve_to_built_in(), Some(BuiltIn::AbiDecode))
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee {
            return Self::MemberBuiltinCall(access.clone());
        }
        let Expression::Identifier(identifier) = callee else {
            unreachable!("unsupported callee expression");
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            unreachable!("callee '{}' does not resolve to a function", identifier.name());
        };
        Self::IdentifierFunctionCall(function_definition)
    }
}

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

    /// Emits a direct, named function call, returning all of its result values in declaration order.
    fn emit_function_call(
        &self,
        function_definition: &FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let (mlir_name, argument_values, return_types, block) =
            self.emit_call_setup(function_definition, arguments, block);
        let results = Function::call(
            mlir_name,
            &argument_values,
            return_types,
            self.expression_context.state,
            &block,
        )
        .expect("function call resolves to a registered signature");
        BlockAnd {
            block,
            value: results,
        }
    }

    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: melior::ir::Type<'context>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let struct_address = AstValue::malloc(AstType::new(result_type), context, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                context,
            );
            let index_value = AstValue::constant(
                index as i64,
                AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_X64),
                context,
                &block,
            );
            let field_address = Pointer::from(struct_address).gep(
                index_value,
                AstType::new(field_type),
                context,
                &block,
            );

            let BlockAnd {
                value: argument_value,
                block: next_block,
            } = argument.emit(self.expression_context, block);
            block = next_block;
            let stored = TypeConversion::from_target_type(field_type, context)
                .emit(argument_value, context, &block);
            field_address.store(AstValue::new(stored), context, &block);
        }

        (struct_address.into_mlir(), block)
    }

    /// Emits argument values for a named call, resolves the callee's MLIR
    /// signature, and casts each argument to its declared parameter type.
    fn emit_call_setup<'a>(
        &'a self,
        function_definition: &FunctionDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [melior::ir::Type<'context>],
        BlockRef<'context, 'block>,
    ) {
        let mut argument_values = Vec::new();
        let mut current_block = block;
        for argument in arguments.iter() {
            let BlockAnd { value, block: next } = argument.emit(self.expression_context, current_block);
            argument_values.push(value);
            current_block = next;
        }

        let (mlir_name, parameter_types, return_types) = self
            .expression_context
            .state
            .resolve_function(function_definition.node_id())
            .expect("callee resolves to a registered signature");

        let context = self.expression_context.state;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, context);
            *value = conversion.emit(*value, context, &current_block);
        }

        (mlir_name, argument_values, return_types, current_block)
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
