//!
//! External calls to a contract-instance method.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits an external contract-instance method call `c.foo(args)`, returning all of its result
    /// values in declaration order.
    ///
    /// The receiver evaluates to the callee's address, the arguments coerce to the callee's declared
    /// parameter types, and `sol.ext_call` encodes the selector and ABI-encoded arguments before
    /// decoding the returns. A `{value: v}` / `{gas: g}` option forwards the wei and gas; a `view` or
    /// `pure` callee lowers to a `STATICCALL`. The call reverts on failure, so the status is discarded.
    pub(super) fn emit_external_member_call(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        arguments: &ArgumentsDeclaration,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let (_status, results, block) = self.emit_external_member_call_fallible(
            access,
            function_definition,
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

    /// Emits the external contract-instance method call `c.foo(args)`, returning its success status,
    /// its result values in declaration order, and the continuation block.
    ///
    /// `try_call` surfaces the success status for a `try`/`catch` guard; without it the `sol.ext_call`
    /// reverts on failure.
    pub(super) fn emit_external_member_call_fallible(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
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
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function_definition, context);
        let callee_name = FunctionEmitter::mlir_function_name(function_definition);
        let selector = function_definition
            .compute_selector()
            .expect("an external member call resolves to a callee with an ABI selector");
        let is_static = matches!(
            function_definition.mutability(),
            FunctionMutability::View | FunctionMutability::Pure
        );

        let BlockAnd {
            value: receiver,
            block,
        } = access.operand().emit(self.expression_context, block);

        let parameter_ids: Vec<NodeId> = function_definition
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();
        let ordered_arguments = arguments
            .ordered_by(&parameter_ids)
            .expect("slang matches every external call argument to a parameter");
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
}
