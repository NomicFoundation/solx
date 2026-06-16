//!
//! Internal / external library call emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::FunctionType;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::ExtCallOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::member_call_kind::MemberCallKind;

impl MemberCallKind {
    /// Emits an internal (`Library { external: false }`) library call — inlined
    /// like an ordinary internal function.
    pub fn emit_library_call<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
        library_function: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let function = context.state.resolve_function(library_function.node_id());
        // A `using for` receiver (`x.f(args)`) is a value and becomes the
        // implicit `self` — the function's first parameter; a namespace qualifier
        // — a library (`L.f`) or import alias (`M.f`) — is not a value, so only
        // the explicit arguments pass.
        if MemberCallKind::is_namespace_qualifier(&access.operand()) {
            let (argument_values, current_block) = context.emit_coerced_arguments(
                positional_arguments,
                &function.parameter_types,
                block,
            );
            let results = function.call(&argument_values, &context.state.builder, &current_block);
            return (results, current_block);
        }

        // Using-for: evaluate the receiver as the leading `self` argument, coerce
        // it to the first parameter, and coerce the explicit arguments to the
        // rest.
        let (parameter_self, parameter_rest) = function
            .parameter_types
            .split_first()
            .expect("a using-for library function has a self parameter");
        let BlockAnd {
            value: self_value,
            block: current_block,
        } = access.operand().emit(context, block);
        let builder = &context.state.builder;
        let self_value = self_value
            .cast(AstType::new(*parameter_self), builder, &current_block)
            .into_mlir();
        let (mut argument_values, current_block) =
            context.emit_coerced_arguments(positional_arguments, parameter_rest, current_block);
        argument_values.insert(0, self_value);
        let results = function.call(&argument_values, &context.state.builder, &current_block);
        (results, current_block)
    }

    /// Emits an external (`Library { external: true }`) library call — a
    /// `delegatecall` to the deployed library via the native `sol.ext_call`
    /// (with `delegate_call` + `library_call` flags), whose conversion owns the ABI
    /// encode, the delegatecall, the revert-bubble, and the result decode. The
    /// library address is a `sol.lib_addr` link placeholder; a `using for`
    /// receiver becomes the implicit leading `self` argument.
    pub fn emit_library_external_call<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        library_name: &solx_utils::ContractName,
        function: &FunctionDefinition,
        arguments: &[Expression],
        self_receiver: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let (parameter_types, return_types) = AstType::resolve_signature(
            function,
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        let selector = function
            .compute_selector()
            .expect("an external library function has a selector");
        let mlir_name = FunctionEmitter::mlir_function_name(function);

        let (argument_values, current_block) = match self_receiver {
            Some(receiver) => {
                let (parameter_self, parameter_rest) = parameter_types
                    .split_first()
                    .expect("a using-for library function has a self parameter");
                let BlockAnd {
                    value: self_value,
                    block,
                } = receiver.emit(context, block);
                let builder = &context.state.builder;
                let self_value = self_value
                    .cast(AstType::new(*parameter_self), builder, &block)
                    .into_mlir();
                let (mut rest_values, block) =
                    context.emit_coerced_argument_expressions(arguments, parameter_rest, block);
                rest_values.insert(0, self_value);
                (rest_values, block)
            }
            None => context.emit_coerced_argument_expressions(arguments, &parameter_types, block),
        };

        let builder = &context.state.builder;
        let address = AstValue::library_address(library_name, builder, &current_block).into_mlir();
        let callee_type = FunctionType::new(builder.context, &parameter_types, &return_types);
        let gas = AstValue::gas_left(builder, &current_block).into_mlir();
        let value = AstValue::constant(
            0,
            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            &current_block,
        )
        .into_mlir();
        let selector_value = AstValue::constant(
            i64::from(selector),
            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            &current_block,
        )
        .into_mlir();
        // `sol.ext_call` yields the `i1` success status (result 0) then the
        // decoded outs; its conversion reverts internally on failure, so the
        // status is dropped and only the decoded results return.
        let operation = current_block.append_operation(sol_op_build!(
            builder,
            ExtCallOperation
                .callee(StringAttribute::new(builder.context, &mlir_name))
                .ins(&argument_values)
                .addr(address)
                .gas(gas)
                .val(value)
                .selector(selector_value)
                .delegate_call(Attribute::unit(builder.context))
                .library_call(Attribute::unit(builder.context))
                .callee_type(TypeAttribute::new(callee_type.into()))
                .status(AstType::signless(
                    builder.context,
                    solx_utils::BIT_LENGTH_BOOLEAN
                ))
                .outs(&return_types)
        ));
        let results = (0..return_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces the declared results")
                    .into()
            })
            .collect();
        (results, current_block)
    }
}
