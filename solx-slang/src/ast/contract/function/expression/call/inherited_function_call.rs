//!
//! Calls redirected through `super` or a base-contract qualifier.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Function;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a `super.f(args)` / `Base.f(args)` call redirected by inherited dispatch to `target_id`,
    /// returning all of its result values in declaration order.
    ///
    /// The arguments are ordered against the lexically named function's parameters, then cast to the
    /// redirected target's registered parameter types, so a named-argument `super.f({b: .., a: ..})`
    /// reaches the target in declaration order.
    pub(super) fn emit_inherited_function_call(
        &self,
        access: &MemberAccessExpression,
        target_id: NodeId,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let context = self.expression_context.state;
        let Some(Definition::Function(function_definition)) =
            access.member().resolve_to_definition()
        else {
            unreachable!("a super/base call resolves its member to a function");
        };
        let parameter_ids: Vec<NodeId> = function_definition
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();

        let (mlir_name, parameter_types, return_types) = context
            .resolve_function(target_id)
            .expect("a super/base call resolves to a registered signature");

        let (argument_values, block) =
            self.emit_ordered_arguments(arguments, &parameter_ids, parameter_types, block);

        let results = Function::call(mlir_name, &argument_values, return_types, context, &block)
            .expect("a super/base call resolves to a registered signature");
        BlockAnd {
            value: results,
            block,
        }
    }
}
