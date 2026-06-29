//!
//! Variable declaration statement emission.
//!

use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;

statement_emit!(VariableDeclarationStatement; |node, context, block| {
    match node.target() {
        VariableDeclarationTarget::SingleTypedDeclaration(single_typed_declaration) => {
            single_typed_declaration.emit(context, block)
        }
        VariableDeclarationTarget::MultiTypedDeclaration(multi_typed_declaration) => {
            multi_typed_declaration.emit(context, block)
        }
    }
});

statement_emit!(SingleTypedDeclaration; |node, context, block| {
    let slang_declared_type = node.declaration().get_type().expect("slang validated");
    let declared_type = AstType::resolve(
        &slang_declared_type,
        LocationPolicy::Declared(None),
        context.state,
    );

    let emitter = ExpressionContext::from(&*context);

    let (block, initial_value) = if let Some(ref initializer_expression) = node.value() {
        let BlockAnd { value, block } =
            initializer_expression.emit_as(declared_type, &emitter, block);
        (block, Some(value.into_mlir()))
    } else {
        (block, None)
    };

    let pointer = if let Some(value) = initial_value {
        let pointer = Pointer::stack(
            AstType::new(declared_type),
            context.state,
            &block,
        );
        pointer.store(AstValue::new(value), context.state, &block);
        pointer.into_mlir()
    } else {
        Pointer::default_initialized(
            AstType::new(declared_type),
            context.state,
            &block,
        )
        .into_mlir()
    };

    context
        .environment
        .define_variable(node.declaration().node_id(), pointer);
    Some(block)
});

statement_emit!(MultiTypedDeclaration; |node, context, block| {
    let expression = node.value().unwrap_parentheses();
    let elements = node.elements();

    let emitter = ExpressionContext::from(&*context);

    let (values, current) = match &expression {
        Expression::TupleExpression(tuple) => {
            let items = tuple.items();
            let mut values = Vec::with_capacity(items.len());
            let mut current = block;
            for item in items.iter() {
                let inner = item
                    .expression()
                    .expect("slang validated");
                let BlockAnd { value, block: next } = inner.emit(&emitter, current);
                values.push(value.into_mlir());
                current = next;
            }
            (values, current)
        }
        Expression::FunctionCallExpression(call) => {
            let BlockAnd { value, block } = call.emit(&emitter, block);
            (value, block)
        }
        Expression::ConditionalExpression(conditional) => {
            let BlockAnd { value, block } = conditional.emit(&emitter, block);
            (value, block)
        }
        _ => unreachable!(
            "tuple deconstruction with this right-hand side shape is not yet supported"
        ),
    };

    for (member, value) in elements.iter().zip(values) {
        let Some(declaration) = member.member() else {
            continue;
        };
        let state = context.state;
        let declared_type = AstType::resolve(
            &declaration.get_type().expect("slang validated"),
            LocationPolicy::Declared(None),
            state,
        );
        let cast = AstValue::from(value).cast(
            AstType::new(declared_type),
            state,
            &current,
        );
        let pointer =
            Pointer::stack(AstType::new(declared_type), state, &current);
        pointer.store(cast, state, &current);
        context
            .environment
            .define_variable(declaration.node_id(), pointer.into_mlir());
    }

    Some(current)
});
