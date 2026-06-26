//!
//! Identifier expression emission: a bare name resolved to its definition.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::StateVariableMutability;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(Identifier; |node, context, block| {
    match node.resolve_to_definition() {
        Some(Definition::StateVariable(state_variable)) => {
            let declared_type = state_variable.get_type().expect("slang validated");
            let element_type = AstType::resolve(
                &declared_type,
                LocationPolicy::Declared(None),
                &context.state.builder,
            );
            if matches!(
                state_variable.mutability(),
                StateVariableMutability::Constant
            ) {
                let initializer = state_variable.value().expect("slang validated");
                if let Expression::StringExpression(string_literal) = &initializer {
                    string_literal.emit_as(element_type, context, block)
                } else {
                    initializer.emit(context, block)
                }
            } else {
                let slot = context
                    .storage_layout
                    .get(&state_variable.node_id())
                    .unwrap_or_else(|| {
                        unreachable!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let value = slot.load(&context.state.builder, element_type, &block);
                BlockAnd {
                    block,
                    value: value.into(),
                }
            }
        }
        Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
            if let Some(value) = context.environment.value_binding(definition.node_id()) {
                return BlockAnd {
                    block,
                    value: AstValue::new(value),
                };
            }
            let pointer =
                Pointer::new(context.environment.variable(definition.node_id()));
            let value = pointer.load(pointer.pointee(), &context.state.builder, &block);
            BlockAnd { block, value }
        }
        Some(Definition::Constant(constant)) => {
            let initializer = constant
                .value()
                .expect("slang validated");
            initializer.emit(context, block)
        }
        Some(Definition::Function(function_definition)) => {
            let target_id = context
                .state
                .resolve_virtual(function_definition.node_id());
            let value = context
                .state
                .resolve_function(target_id)
                .pointer_constant(&context.state.builder, &block);
            BlockAnd { block, value }
        }
        Some(Definition::Library(library)) => {
            let name = solx_utils::ContractName::new(
                library.get_file_id().to_owned(),
                Some(library.name().name()),
            );
            let value = AstValue::library_address(&name, &context.state.builder, &block);
            BlockAnd { block, value }
        }
        None => unreachable!("slang resolves every identifier reference"),
        Some(other) => {
            unreachable!("unsupported identifier reference {:?}", other.node_id())
        }
    }
});
