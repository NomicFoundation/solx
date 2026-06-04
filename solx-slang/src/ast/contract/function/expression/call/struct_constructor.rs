//!
//! Struct constructor call lowering: `S(a, b, …)`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to lower `S(a, b, …)` as a positional struct constructor.
    ///
    /// Returns `Ok(None)` when the callee is not a struct name, so the caller
    /// falls through. Named-argument construction (`S({a: x})`) defers to a
    /// later domain.
    pub fn try_emit_struct_constructor(
        &self,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::Identifier(identifier) = call.operand() else {
            return Ok(None);
        };
        let Some(Definition::Struct(struct_definition)) = identifier.resolve_to_definition() else {
            return Ok(None);
        };
        let builder = &self.expression_emitter.state.builder;
        let result_slang_type = call
            .get_type()
            .expect("the binder types a struct constructor");
        let result_type = TypeConversion::resolve_slang_type(&result_slang_type, None, builder);
        let (value, block) =
            self.emit_struct_constructor(&struct_definition, result_type, arguments, block)?;
        Ok(Some((Some(value), block)))
    }

    /// Allocates a fresh memory struct and stores each positional argument into
    /// its field via `sol.malloc` + per-field `sol.gep` + `sol.store`.
    ///
    /// Reference-typed fields (nested struct / array) take their argument as a
    /// reference and are deep-copied with `sol.copy`; value-typed fields cast
    /// and store.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let field_slang_type = member
                .get_type()
                .expect("the binder types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                builder,
            );
            let index_value = builder.emit_sol_constant(index as i64, builder.types.ui64, &block);
            let field_address =
                builder.emit_sol_gep(struct_address, index_value, field_type, &block);

            let (argument_value, next_block) =
                self.expression_emitter.emit_value(&argument, block)?;
            block = next_block;
            if field_slang_type.is_reference_type() {
                builder.emit_sol_copy(argument_value, field_address, &block);
            } else {
                let stored = TypeConversion::from_target_type(field_type, builder).emit(
                    argument_value,
                    builder,
                    &block,
                );
                builder.emit_sol_store(stored, field_address, &block);
            }
        }
        Ok((struct_address, block))
    }
}
