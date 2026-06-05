//!
//! Library calls — internal and external.
//!
//! Internal (no-selector) library functions — `L.f(args)`, `using`-for
//! `x.f(args)`, and bare sibling calls — are inlined into the contract module
//! (see `contract::library::collect_library_functions`) and called through a
//! plain internal `sol.call` to their registered symbol, with a `using`-for
//! value receiver prepended as the implicit `self`.
//!
//! External / public (selector-carrying) library functions are linked
//! separately and called by `delegatecall` to the library object's address
//! (`sol.lib_addr`): the calldata is the 4-byte selector followed by the
//! ABI-encoded arguments, and the callee's revert data is bubbled on failure.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to lower an internal library call `L.f(args)` / `x.f(args)` whose
    /// member resolves to a collected (inlined) library function, returning every
    /// result in declaration order. Returns `Ok(None)` when the callee is not a
    /// member access onto such a function, so the caller falls through.
    ///
    /// A `using`-for value receiver (`x.f(args)`) is the implicit first argument
    /// (`self`); a namespace qualifier (`L.f(args)` / `import as M; M.f(args)`)
    /// is not a value and contributes no `self`.
    pub fn try_emit_library_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::MemberAccessExpression(access) = callee else {
            return Ok(None);
        };
        let Some(Definition::Function(library_function)) = access.member().resolve_to_definition()
        else {
            return Ok(None);
        };
        if !self
            .expression_emitter
            .state
            .library_function_ids
            .contains(&library_function.node_id())
        {
            return Ok(None);
        }

        // A namespace qualifier (`L.f` / import alias) is not a value, so it
        // contributes no `self`; a value receiver (`x.f`) becomes the implicit
        // first argument.
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
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(library_function.node_id())?;

        let mut argument_values = Vec::new();
        let mut current_block = block;
        if !receiver_is_qualifier {
            let (self_value, next) = self
                .expression_emitter
                .emit_value(&access.operand(), current_block)?;
            argument_values.push(self_value);
            current_block = next;
        }
        for argument in arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }

        let builder = &self.expression_emitter.state.builder;
        self.coerce_arguments(&mut argument_values, parameter_types, &current_block);
        let results = builder.emit_sol_call_results(
            mlir_name,
            &argument_values,
            return_types,
            &current_block,
        )?;
        Ok(Some((results, current_block)))
    }

    /// Tries to lower an external / public library call `L.f(args)` or a
    /// `using`-for `x.f(args)` onto a selector-carrying library function, as a
    /// `delegatecall` to the linked library. Returns `Ok(None)` when the callee
    /// is not such a call, so the caller falls through.
    ///
    /// A direct `L.f(args)` resolves the enclosing library from the qualifier;
    /// a value-receiver `x.f(args)` looks the enclosing library up by definition
    /// id in `library_function_symbols` (slang exposes no enclosing-library
    /// accessor on a resolved function) and prepends the receiver as `self`.
    pub fn try_emit_library_external_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        // `L.f(args)` — a library qualifier with a public/external member.
        if let Expression::MemberAccessExpression(access) = callee
            && let Expression::Identifier(operand) = access.operand()
            && let Some(Definition::Library(library)) = operand.resolve_to_definition()
            && let Some(Definition::Function(function)) = access.member().resolve_to_definition()
            && matches!(
                function.visibility(),
                FunctionVisibility::External | FunctionVisibility::Public
            )
            && function.compute_selector().is_some()
        {
            // The linker symbol is the fully-qualified `file:Library` name,
            // matching solc so `link_references` round-trips.
            let library_symbol = format!("{}:{}", library.get_file_id(), library.name().name());
            return self
                .emit_library_external_call(&library_symbol, &function, arguments, None, block)
                .map(Some);
        }

        // `using D for T; x.f(args)` — a value receiver attaching a
        // selector-carrying public/external library function. A namespace
        // operand (`L.f` / `M.f` / `C.f`) or `this`/`super` is not a value
        // receiver, so exclude those (handled above or as contract dispatch).
        if let Expression::MemberAccessExpression(access) = callee
            && !matches!(
                access.operand(),
                Expression::Identifier(identifier)
                    if matches!(
                        identifier.resolve_to_definition(),
                        Some(
                            Definition::Library(_)
                                | Definition::Contract(_)
                                | Definition::Import(_)
                                | Definition::ImportedSymbol(_)
                        )
                    )
            )
            && !matches!(
                access.operand(),
                Expression::ThisKeyword(_) | Expression::SuperKeyword(_)
            )
            && let Some(Definition::Function(function)) = access.member().resolve_to_definition()
            && function.compute_selector().is_some()
            && let Some(library_symbol) = self
                .expression_emitter
                .state
                .library_function_symbols
                .get(&function.node_id())
                .cloned()
        {
            let receiver = access.operand();
            return self
                .emit_library_external_call(
                    &library_symbol,
                    &function,
                    arguments,
                    Some(&receiver),
                    block,
                )
                .map(Some);
        }

        Ok(None)
    }

    /// Emits an external/public library call as a `delegatecall` to the linked
    /// library `library_name`: ABI-encode `(selector, args)`, `sol.lib_addr` for
    /// the address, `sol.bare_delegate_call`, re-revert (bubbling the callee's
    /// revert data) on failure, then decode the return value. A `using`-for
    /// `self_receiver` is prepended as the implicit first argument.
    fn emit_library_external_call(
        &self,
        library_name: &str,
        function: &FunctionDefinition,
        arguments: &PositionalArguments,
        self_receiver: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            function,
            &self.expression_emitter.state.builder,
        );
        let selector = function
            .compute_selector()
            .expect("external library function has a selector");

        // A `using`-for value receiver is the implicit `self` first argument;
        // the explicit arguments follow. Coercion aligns every value with its
        // declared parameter type by position.
        let mut argument_values = Vec::new();
        let mut current_block = block;
        if let Some(receiver) = self_receiver {
            let (value, next) = self
                .expression_emitter
                .emit_value(receiver, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        for argument in arguments.iter() {
            let (value, next) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next;
        }
        self.coerce_arguments(&mut argument_values, &parameter_types, &current_block);

        let builder = &self.expression_emitter.state.builder;
        // Calldata: the 4-byte selector followed by the ABI-encoded arguments.
        let selector_unsigned = builder.emit_sol_constant(
            i64::from(selector),
            Type::from(IntegerType::unsigned(builder.context, 32)),
            &current_block,
        );
        let selector_bytes = builder.emit_sol_cast(
            selector_unsigned,
            builder.types.fixed_bytes(4),
            &current_block,
        );
        let calldata = builder.emit_sol_encode(
            &argument_values,
            Some(selector_bytes),
            false,
            &current_block,
        );
        let address = builder.emit_sol_lib_addr(library_name, &current_block);
        let (status, return_data) =
            builder.emit_sol_bare_delegate_call(address, calldata, &current_block);

        // Bubble the callee's revert data when the delegatecall failed.
        let (then_block, else_block) = builder.emit_sol_if(status, &current_block);
        builder.emit_sol_yield(&then_block);
        builder.emit_revert_returndata(&else_block);
        builder.emit_sol_yield(&else_block);

        if return_types.is_empty() {
            return Ok((None, current_block));
        }
        let decoded = builder.emit_sol_decode(return_data, &return_types, &current_block);
        Ok((Some(decoded[0]), current_block))
    }
}
