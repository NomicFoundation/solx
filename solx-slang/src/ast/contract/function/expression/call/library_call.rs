//!
//! Internal / external library call lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::r#type::FunctionType;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits an internal (`Library { external: false }`) library call — inlined
    /// like an ordinary internal function.
    pub fn emit_library_call(
        &self,
        access: &MemberAccessExpression,
        library_function: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(library_function.node_id())?;
        // A `using for` receiver (`x.f(args)`) is a value and becomes the
        // implicit `self` — the function's first parameter; a namespace qualifier
        // — a library (`L.f`) or import alias (`M.f`) — is not a value, so only
        // the explicit arguments pass.
        let receiver_is_qualifier = matches!(
            access.operand(),
            Expression::Identifier(identifier)
                if matches!(
                    identifier.resolve_to_definition(),
                    Some(
                        Definition::Library(_)
                            | Definition::Import(_)
                            | Definition::ImportedSymbol(_)
                    )
                )
        );

        if receiver_is_qualifier {
            let (argument_values, current_block) =
                self.emit_coerced_arguments(positional_arguments, parameter_types, block)?;
            let results = self
                .expression_emitter
                .state
                .builder
                .emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
            return Ok((results, current_block));
        }

        // Using-for: evaluate the receiver as the leading `self` argument, coerce
        // it to the first parameter, and coerce the explicit arguments to the
        // rest.
        let (parameter_self, parameter_rest) = parameter_types
            .split_first()
            .expect("a using-for library function has a self parameter");
        let (self_value, current_block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let builder = &self.expression_emitter.state.builder;
        let self_value = TypeConversion::from_target_type(*parameter_self, builder).emit(
            self_value,
            builder,
            &current_block,
        );
        let (mut argument_values, current_block) =
            self.emit_coerced_arguments(positional_arguments, parameter_rest, current_block)?;
        argument_values.insert(0, self_value);
        let results = self
            .expression_emitter
            .state
            .builder
            .emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
        Ok((results, current_block))
    }

    /// Emits an external (`Library { external: true }`) library call — a
    /// `delegatecall` to the deployed library via the native `sol.ext_call`
    /// (with `delegate_call` + `library_call` flags), whose lowering owns the ABI
    /// encode, the delegatecall, the revert-bubble, and the result decode. The
    /// library address is a `sol.lib_addr` link placeholder; a `using for`
    /// receiver becomes the implicit leading `self` argument.
    pub fn emit_library_external_call(
        &self,
        library_name: &str,
        function: &FunctionDefinition,
        arguments: &PositionalArguments,
        self_receiver: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            function,
            &self.expression_emitter.state.builder,
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
                let (self_value, block) = self.expression_emitter.emit_value(receiver, block)?;
                let builder = &self.expression_emitter.state.builder;
                let self_value = TypeConversion::from_target_type(*parameter_self, builder)
                    .emit(self_value, builder, &block);
                let (mut rest_values, block) =
                    self.emit_coerced_arguments(arguments, parameter_rest, block)?;
                rest_values.insert(0, self_value);
                (rest_values, block)
            }
            None => self.emit_coerced_arguments(arguments, &parameter_types, block)?,
        };

        let builder = &self.expression_emitter.state.builder;
        let address = builder.emit_sol_lib_addr(library_name, &current_block);
        let callee_type = FunctionType::new(builder.context, &parameter_types, &return_types);
        let results = builder.emit_sol_ext_call_library(
            &mlir_name,
            &argument_values,
            address,
            selector,
            callee_type,
            &current_block,
        );
        Ok((results, current_block))
    }

    /// Re-raises a bubbled revert (`returndatacopy` + `revert`). Oracle free
    /// assoc fn → `&self` method (Rule-5).
    pub fn emit_bubble_revert(&self, block: &BlockRef<'context, 'block>) {
        let _ = block;
        unimplemented!("bubble revert")
    }
}
