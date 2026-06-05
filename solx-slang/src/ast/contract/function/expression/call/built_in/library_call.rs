//!
//! Internal library calls — `L.f(args)` and `using`-for `x.f(args)` for a
//! no-selector (internal / private, or non-ABI-encodable `public`) library
//! function. The function body is inlined into the contract module (see
//! `contract::library::collect_library_functions`), so the call lowers to a
//! plain internal `sol.call` to its registered symbol — with the `using`-for
//! value receiver prepended as the implicit `self` argument.
//!
//! External / public library functions (which carry a selector) are dispatched
//! by delegatecall in a separate lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

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
}
