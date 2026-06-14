//! Variable declaration statement emission.

use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;
use crate::ast::type_conversion::TypeConversion;

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
            slang_type.resolve_type(LocationPolicy::Declared(None), &context.state.builder)
        })
        .unwrap_or_else(|| {
            crate::ast::Type::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
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
        } = (Toward {
            expression: initializer_expression,
            target_type: declared_type,
        })
        .emit(&emitter, block)?;
        let cast_value = initial_value
            .coerce_to(
                crate::ast::Type::new(declared_type),
                &context.state.builder,
                &block,
            )
            .into_mlir();
        (block, Some(cast_value))
    } else {
        (block, None)
    };

    let pointer = if let Some(value) = initial_value {
        let pointer = crate::ast::Pointer::stack_slot(
            crate::ast::Type::new(declared_type),
            &context.state.builder,
            &block,
        );
        pointer.store(crate::ast::Value::new(value), &context.state.builder, &block);
        pointer.into_mlir()
    } else {
        // No initializer: default-initialise the slot to the type's zero
        // through the shared primitive (memory aggregates malloc'd, empty
        // `string`/`bytes` a plain malloc, scalar value types their own
        // zero, integers a zeroed slot, references a bare slot).
        TypeConversion::emit_default_initialized_slot(
            slang_declared_type.as_ref(),
            declared_type,
            &context.state.builder,
            &block,
        )
    };

    context
        .environment
        .define_variable(node.declaration().node_id(), pointer);
    Ok(Some(block))
});

statement_emit!(MultiTypedDeclaration; |node, context, block| {
    let expression = node.value();
    let elements = node.elements();

    let emitter = ExpressionContext::from(&*context);

    let (values, current) = match &expression {
        Expression::TupleExpression(tuple) => {
            let items = tuple.items();
            assert!(
                items.len() == elements.len(),
                "tuple deconstruction arity mismatch: {} LHS slots vs {} RHS values",
                elements.len(),
                items.len(),
            );
            let mut values = Vec::with_capacity(items.len());
            let mut current = block;
            for item in items.iter() {
                let inner = item
                    .expression()
                    .expect("a deconstruction RHS tuple element has an inner expression");
                let BlockAnd { value, block: next } = inner.emit(&emitter, current)?;
                values.push(value.into_mlir());
                current = next;
            }
            (values, current)
        }
        Expression::FunctionCallExpression(call) => {
            let (values, current) = emitter.emit_function_call_results(call, block)?;
            assert!(
                values.len() == elements.len(),
                "tuple deconstruction arity mismatch: {} LHS slots vs {} call results",
                elements.len(),
                values.len(),
            );
            (values, current)
        }
        Expression::ConditionalExpression(conditional) => {
            // `(a, b) = cond ? (x, y) : (z, w)` — the conditional yields one
            // value per tuple element via the shared tuple-conditional path.
            let (values, current) = emitter.emit_conditional_tuple_values(conditional, block)?;
            assert!(
                values.len() == elements.len(),
                "tuple deconstruction arity mismatch: {} LHS slots vs {} conditional values",
                elements.len(),
                values.len(),
            );
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
            .map(|slang_type| slang_type.resolve_type(LocationPolicy::Declared(None), builder))
            .unwrap_or_else(|| {
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            });
        let cast = crate::ast::Value::from(value).coerce_to(
            crate::ast::Type::new(declared_type),
            builder,
            &current,
        );
        let pointer =
            crate::ast::Pointer::stack_slot(crate::ast::Type::new(declared_type), builder, &current);
        pointer.store(cast, builder, &current);
        context
            .environment
            .define_variable(declaration.node_id(), pointer.into_mlir());
    }

    Ok(Some(current))
});
