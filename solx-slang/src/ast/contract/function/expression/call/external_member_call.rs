//!
//! External calls to a contract-instance method or `public` state-variable getter.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;

use solx_mlir::Context;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::getter::signature::Signature;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits an external call to a contract-instance method or `public` state-variable getter
    /// `c.foo(args)`, returning all of its result values in declaration order.
    ///
    /// The receiver evaluates to the callee's address, the arguments coerce to the callee's declared
    /// parameter types, and `sol.ext_call` encodes the selector and ABI-encoded arguments before
    /// decoding the returns. A `{value: v}` / `{gas: g}` option forwards the wei and gas; a `view` or
    /// `pure` callee lowers to a `STATICCALL`. The call reverts on failure, so the status is discarded.
    pub(super) fn emit_external_member_call(
        &self,
        access: &MemberAccessExpression,
        definition: &Definition,
        arguments: &ArgumentsDeclaration,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let (_status, results, block) = self.emit_external_member_call_fallible(
            access,
            definition,
            arguments,
            call_value,
            call_gas,
            false,
            block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }

    /// Emits the external call `c.foo(args)`, returning its success status, its result values in
    /// declaration order, and the continuation block.
    ///
    /// `try_call` surfaces the success status for a `try`/`catch` guard; without it the `sol.ext_call`
    /// reverts on failure.
    pub(super) fn emit_external_member_call_fallible(
        &self,
        access: &MemberAccessExpression,
        definition: &Definition,
        arguments: &ArgumentsDeclaration,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        try_call: bool,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let context = self.expression_context.state;
        let (callee_name, selector, parameter_types, return_types, is_static) =
            Self::resolve_external_callee(definition, context);

        let BlockAnd {
            value: receiver,
            block,
        } = access.operand().emit(self.expression_context, block);

        let ordered_arguments = Self::ordered_arguments(definition, arguments);
        let mut argument_values = Vec::with_capacity(parameter_types.len());
        let mut block = block;
        for (argument, &parameter_type) in ordered_arguments.iter().zip(&parameter_types) {
            let BlockAnd { value, block: next } = argument.emit(self.expression_context, block);
            let value =
                TypeConversion::from_target_type(parameter_type, context).emit(value, context, &next);
            argument_values.push(value);
            block = next;
        }

        let (status, results) = AstValue::external_call(
            AstValue::new(receiver),
            &callee_name,
            selector,
            &parameter_types,
            &argument_values,
            &return_types,
            call_value,
            call_gas,
            is_static,
            try_call,
            context,
            &block,
        );
        (status, results, block)
    }

    /// Resolves the callee's MLIR name, ABI selector, Sol-typed parameter and result types, and
    /// whether the call is to a read-only callee.
    fn resolve_external_callee(
        definition: &Definition,
        context: &Context<'context>,
    ) -> (String, u32, Vec<Type<'context>>, Vec<Type<'context>>, bool) {
        match definition {
            Definition::Function(function_definition) => {
                let (parameter_types, return_types) =
                    TypeConversion::resolve_function_types(function_definition, context);
                (
                    FunctionEmitter::mlir_function_name(function_definition),
                    function_definition
                        .compute_selector()
                        .expect("an external member call resolves to a callee with an ABI selector"),
                    parameter_types,
                    return_types,
                    matches!(
                        function_definition.mutability(),
                        FunctionMutability::View | FunctionMutability::Pure
                    ),
                )
            }
            Definition::StateVariable(state_variable) => {
                let (parameter_types, return_types) = state_variable
                    .getter_signature(context)
                    .expect("a public accessor with no returnable members is invalid");
                (
                    state_variable
                        .compute_canonical_signature()
                        .expect("a public accessor has a canonical signature"),
                    state_variable
                        .compute_selector()
                        .expect("a public accessor has a selector"),
                    parameter_types,
                    return_types,
                    true,
                )
            }
            _ => unreachable!("an external member call resolves to a function or state variable"),
        }
    }

    /// The call's argument expressions in ABI order: a function orders them against its parameters,
    /// a getter takes the mapping keys and array indices positionally.
    fn ordered_arguments(
        definition: &Definition,
        arguments: &ArgumentsDeclaration,
    ) -> Vec<Expression> {
        match definition {
            Definition::Function(function_definition) => {
                let parameter_ids: Vec<_> = function_definition
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                arguments
                    .ordered_by(&parameter_ids)
                    .expect("slang matches every external call argument to a parameter")
            }
            Definition::StateVariable(_) => {
                let ArgumentsDeclaration::PositionalArguments(positional) = arguments else {
                    unreachable!("a public accessor is called with positional keys");
                };
                positional.iter().collect()
            }
            _ => unreachable!("an external member call resolves to a function or state variable"),
        }
    }
}
