//!
//! Identifier expression lowering: a bare name resolved to its definition.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;
use solx_mlir::VariableBinding;
use solx_mlir::ods::sol::LibAddrOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LibraryExt;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(Identifier; |node, context, block| {
    // A bare name resolves to exactly one definition (slang's binder is total),
    // and each definition kind reads differently: a state variable through
    // storage, a local / parameter through its stack slot, a constant by
    // inlining its initializer, a function as an internal function pointer, and
    // a library name as its linked deploy address.
    match node.resolve_to_definition() {
        Some(Definition::StateVariable(state_variable)) => context
            .emit_state_variable_read(&state_variable, block)
            .map(|(value, block)| BlockAnd {
                block,
                value: value.into(),
            }),
        Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
            let VariableBinding {
                pointer,
                element_type,
            } = context.environment.variable_with_type(definition.node_id());
            let value = context
                .state
                .builder
                .emit_sol_load(pointer, element_type, &block)?;
            Ok(BlockAnd {
                block,
                value: value.into(),
            })
        }
        Some(Definition::Constant(constant)) => {
            let initializer = constant
                .value()
                .expect("a Solidity constant has an initializer");
            initializer.emit(context, block)
        }
        Some(Definition::Function(function_definition)) => context
            .emit_internal_function_pointer(&function_definition, block)
            .map(|(value, block)| BlockAnd {
                block,
                value: value.into(),
            }),
        Some(Definition::Library(library)) => {
            // A library name used as a value (`address(L)`) is its linked deploy
            // address, placed by its link symbol.
            let builder = &context.state.builder;
            let value: Value<'context, 'block> = sol_op!(
                builder,
                &block,
                LibAddrOperation
                    ._name(StringAttribute::new(builder.context, &library.link_symbol()))
                    .val(builder.types.sol_address)
            );
            Ok(BlockAnd {
                block,
                value: value.into(),
            })
        }
        None => unreachable!("slang resolves every identifier reference"),
        Some(other) => {
            unimplemented!("unsupported identifier reference {:?}", other.node_id())
        }
    }
});
