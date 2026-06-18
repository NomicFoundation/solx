//! Variable declaration statement emission.

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
    let slang_declared_type = node.declaration().get_type();
    let declared_type = slang_declared_type
        .as_ref()
        .map(|slang_type| {
            AstType::resolve(slang_type, LocationPolicy::Declared(None), &context.state.builder)
        })
        .unwrap_or_else(|| {
            AstType::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                .into_mlir()
        });

    let emitter = ExpressionContext::from(&*context);

    // For explicit initializers, evaluate and cast before alloca to match
    // solc's emission order (constant → cast → alloca → store).
    // For implicit zero-initialization, alloca is emitted first.
    let (block, initial_value) = if let Some(ref initializer_expression) = node.value() {
        let BlockAnd {
            value: initial_value,
            block,
        } = if let Expression::StringExpression(string_literal) = initializer_expression {
            string_literal.emit_as(declared_type, &emitter, block)
        } else {
            initializer_expression.emit(&emitter, block)
        };
        let cast_value = initial_value
            .cast(
                AstType::new(declared_type),
                &context.state.builder,
                &block,
            )
            .into_mlir();
        (block, Some(cast_value))
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
        // No initializer: default-initialise the slot to the type's zero
        // through the shared primitive (memory aggregates malloc'd, empty
        // `string`/`bytes` a plain malloc, scalar value types their own
        // zero, integers a zeroed slot, references a bare slot).
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
            let (values, current) = call.emit(&emitter, block);
            (values, current)
        }
        Expression::ConditionalExpression(conditional) => {
            // `(a, b) = cond ? (x, y) : (z, w)` — the conditional yields one
            // value per tuple element through its own Emit.
            let (values, current) = conditional.emit(&emitter, block);
            (values, current)
        }
        _ => unimplemented!(
            "tuple deconstruction with this right-hand side shape is not yet supported"
        ),
    };

    for (member, value) in elements.iter().zip(values) {
        let Some(declaration) = member.member() else {
            // Discard the value; nothing to bind.
            continue;
        };
        let builder = &context.state.builder;
        let declared_type = declaration
            .get_type()
            .map(|slang_type| AstType::resolve(&slang_type, LocationPolicy::Declared(None), builder))
            .unwrap_or_else(|| {
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            });
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
