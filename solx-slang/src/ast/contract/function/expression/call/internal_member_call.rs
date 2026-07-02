//!
//! Internal member-function calls: an inlined library member or a `using for` receiver.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Function;

use crate::ast::analysis::query::member_access_operand::MemberAccessOperand;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a member call `L.f(args)` / `x.f(args)` to an internal function, inlined into the calling
    /// module, returning all of its result values in declaration order.
    ///
    /// A namespace-qualified access `L.f(args)` orders its arguments against the callee's parameters.
    /// A `using for` receiver `x.f(args)` evaluates the receiver, casts it to the first parameter
    /// type, and forwards it ahead of the remaining arguments.
    pub(super) fn emit_internal_member_call(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let context = self.expression_context.state;
        let (mlir_name, parameter_types, return_types) = context
            .resolve_function(function_definition.node_id())
            .expect("an internal member call resolves to a registered signature");
        let parameter_ids: Vec<NodeId> = function_definition
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();

        let operand = access.operand();
        let (argument_values, block) =
            if MemberAccessOperand(&operand).is_namespace_qualifier() {
                self.emit_ordered_arguments(arguments, &parameter_ids, parameter_types, block)
            } else {
                let (&receiver_type, rest_types) = parameter_types
                    .split_first()
                    .expect("a `using for` receiver occupies the first parameter");
                let BlockAnd {
                    value: receiver,
                    block,
                } = operand.emit(self.expression_context, block);
                let receiver =
                    TypeConversion::from_target_type(receiver_type, context).emit(receiver, context, &block);
                let (mut argument_values, block) =
                    self.emit_ordered_arguments(arguments, &parameter_ids[1..], rest_types, block);
                argument_values.insert(0, receiver);
                (argument_values, block)
            };

        let results = Function::call(mlir_name, &argument_values, return_types, context, &block)
            .expect("an internal member call resolves to a registered signature");
        BlockAnd {
            value: results,
            block,
        }
    }
}
