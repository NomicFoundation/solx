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
        &context.state.builder,
    );

    let emitter = ExpressionContext::from(&*context);

    // Explicit initializers evaluate and cast before alloca, matching solc's order (cast → alloca → store).
    let (block, initial_value) = if let Some(ref initializer_expression) = node.value() {
        let BlockAnd { value, block } =
            initializer_expression.emit_as(declared_type, &emitter, block);
        (block, Some(value.into_mlir()))
    } else {
        (block, None)
    };

    let pointer = if let Some(value) = initial_value {
        let pointer = Pointer::stack_slot(
            AstType::new(declared_type),
            &context.state.builder,
            &block,
        );
        pointer.store(AstValue::new(value), &context.state.builder, &block);
        pointer.into_mlir()
    } else {
        Pointer::default_initialized(
            AstType::new(declared_type),
            &context.state.builder,
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
    let expression = node.value();
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
            // `(a, b) = cond ? (x, y) : (z, w)` — the conditional yields one
            // value per tuple element through its own Emit.
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
        let builder = &context.state.builder;
        let declared_type = AstType::resolve(
            &declaration.get_type().expect("slang validated"),
            LocationPolicy::Declared(None),
            builder,
        );
        let cast = AstValue::from(value).cast(
            AstType::new(declared_type),
            builder,
            &current,
        );
        let pointer =
            Pointer::stack_slot(AstType::new(declared_type), builder, &current);
        pointer.store(cast, builder, &current);
        context
            .environment
            .define_variable(declaration.node_id(), pointer.into_mlir());
    }

    Some(current)
});
