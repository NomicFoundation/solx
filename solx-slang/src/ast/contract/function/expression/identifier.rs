//!
//! Identifier expression emission: a bare name resolved to its definition.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(Identifier; |node, context, block| {
    let name = node.name();
    match node.resolve_to_definition() {
        Some(Definition::StateVariable(state_variable))
            if matches!(state_variable.mutability(), StateVariableMutability::Constant) =>
        {
            let initializer = state_variable
                .value()
                .expect("a constant state variable is initialised");
            initializer.emit(context, block)
        }
        Some(Definition::StateVariable(state_variable)) => {
            let slot = context
                .storage_layout
                .get(&state_variable.node_id())
                .expect("state variable is registered in the storage layout");
            let declared_type = state_variable
                .get_type()
                .expect("binder types every state variable");
            let element_type =
                TypeConversion::resolve_slang_type(&declared_type, None, context.state);
            let address = Pointer::addr_of(
                &slot.name,
                AstType::new(ExpressionContext::address_type(
                    context.state,
                    element_type,
                    slot.location,
                    &declared_type,
                )),
                context.state,
                &block,
            );
            let value = address
                .load(AstType::new(element_type), context.state, &block)
                .into_mlir();
            BlockAnd { block, value }
        }
        Some(Definition::Variable(_) | Definition::Parameter(_)) => {
            let (pointer, element_type) = context.environment.variable_with_type(&name);
            let value = Pointer::new(pointer)
                .load(AstType::new(element_type), context.state, &block)
                .into_mlir();
            BlockAnd { block, value }
        }
        Some(Definition::Constant(constant)) => {
            let initializer = constant.value().expect("constant has an initializer");
            initializer.emit(context, block)
        }
        Some(Definition::Function(function_definition)) => {
            let value = context
                .state
                .function_signatures
                .get(&function_definition.node_id())
                .expect("bare function name resolves to a registered signature")
                .pointer_constant(context.state, &block)
                .into_mlir();
            BlockAnd { block, value }
        }
        None => unreachable!("slang resolves every identifier reference: {name}"),
        Some(_) => unreachable!("unsupported identifier reference: {name}"),
    }
});
