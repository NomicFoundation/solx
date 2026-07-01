//!
//! Struct construction from a call expression.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_utils::DataLocation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    pub(super) fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let struct_address = AstValue::malloc(AstType::new(result_type), context, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                context,
            );
            let index_value = AstValue::constant(
                index as i64,
                AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_X64),
                context,
                &block,
            );
            let field_address = Pointer::from(struct_address).gep(
                index_value,
                AstType::new(field_type),
                context,
                &block,
            );

            let BlockAnd {
                value: argument_value,
                block: next_block,
            } = argument.emit(self.expression_context, block);
            block = next_block;
            let stored = TypeConversion::from_target_type(field_type, context)
                .emit(argument_value, context, &block);
            field_address.store(AstValue::new(stored), context, &block);
        }

        (struct_address.into_mlir(), block)
    }
}
