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
    // A bare name resolves to exactly one definition (slang's binder is total); each kind reads differently.
    match node.resolve_to_definition() {
        Some(Definition::StateVariable(state_variable)) => {
            // A `constant` inlines its compile-time initializer (no slot); any other state variable loads its slot.
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
                // Emit toward the declared type so a `bytesN constant` initialised
                // from a string literal folds to a fixed-bytes constant.
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
                        unimplemented!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let value = slot.load(&context.state.builder, element_type, &block);
                BlockAnd {
                    block,
                    value: value.into(),
                }
            }
        }
        Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
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
            // A bare function name binds the most-derived override (virtual dispatch); an explicit
            // `Base.f` skips this redirect.
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
            // A library name used as a value (`address(L)`) is its linked deploy
            // address, placed by its link symbol.
            let name = solx_utils::ContractName::new(
                library.get_file_id().to_owned(),
                Some(library.name().name()),
            );
            let value = AstValue::library_address(&name, &context.state.builder, &block);
            BlockAnd { block, value }
        }
        None => unreachable!("slang resolves every identifier reference"),
        Some(other) => {
            unimplemented!("unsupported identifier reference {:?}", other.node_id())
        }
    }
});
