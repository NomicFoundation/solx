//!
//! Struct-literal constructor lowering: `S(a, b, c)`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits a struct-literal constructor `S(a, b, c)` in memory: allocates the
    /// struct, then stores each argument (cast to its field type) at the
    /// field's `sol.gep` address.
    pub fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(positional_arguments.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang types every struct member");
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
            let stored = TypeConversion::from_target_type(field_type, builder).emit(
                argument_value,
                builder,
                &block,
            );
            builder.emit_sol_store(stored, field_address, &block);
        }

        Ok((struct_address, block))
    }
}
