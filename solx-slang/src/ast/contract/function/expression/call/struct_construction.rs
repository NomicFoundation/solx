//!
//! Struct construction from a call expression.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StructDefinition;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A struct construction call.
pub struct StructConstruction {
    /// The original call expression.
    pub call: FunctionCallExpression,
    /// The constructed struct definition.
    pub definition: StructDefinition,
    /// Arguments ordered against struct members.
    pub arguments: CallArguments,
}

impl StructConstruction {
    /// Classifies a function call as struct construction.
    pub fn from_call(call: &FunctionCallExpression, callee: &Expression) -> Option<Self> {
        let definition = match callee {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => None,
        };
        let Some(Definition::Struct(definition)) = definition else {
            return None;
        };
        let member_ids: Vec<NodeId> = definition
            .members()
            .iter()
            .map(|member| member.node_id())
            .collect();
        Some(Self {
            call: call.clone(),
            definition,
            arguments: CallArguments::for_parameter_ids(&call.arguments(), &member_ids),
        })
    }

    /// Emits the struct construction.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let result_type = AstType::resolve_optional(self.call.get_type(), context.state)
            .expect("slang validated");
        let state = context.state;
        let struct_address = AstValue::malloc(result_type, None, false, state, &block).into_mlir();
        let struct_pointer = Pointer::new(struct_address);
        let mut block = block;
        for (index, (member, argument)) in self
            .definition
            .members()
            .iter()
            .zip(self.arguments.expressions.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang validated");
            let field_type = AstType::resolve(
                &field_slang_type,
                LocationPolicy::Declared(Some(solx_utils::DataLocation::Memory)),
                state,
            );
            let index_value = AstValue::constant(
                index as i64,
                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_X64),
                state,
                &block,
            );
            let field_address =
                struct_pointer.gep(index_value, AstType::new(field_type), false, state, &block);
            let BlockAnd {
                value: argument_value,
                block: next_block,
            } = argument.emit(context, block);
            block = next_block;
            let stored = argument_value.cast(AstType::new(field_type), state, &block);
            field_address.store(stored, state, &block);
        }
        BlockAnd {
            value: vec![struct_address],
            block,
        }
    }
}
