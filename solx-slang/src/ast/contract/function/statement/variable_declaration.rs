//! Variable declaration statement lowering.

use melior::ir::BlockLike;
use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;
use solx_mlir::ods::sol::StoreOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a variable declaration with optional initializer.
    pub fn emit_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match declaration.target() {
            VariableDeclarationTarget::SingleTypedDeclaration(single_typed_declaration) => {
                self.emit_single_typed_declaration(&single_typed_declaration, block)
            }
            VariableDeclarationTarget::MultiTypedDeclaration(multi_typed_declaration) => {
                self.emit_multi_typed_declaration(&multi_typed_declaration, block)
            }
        }
    }

    fn emit_single_typed_declaration(
        &mut self,
        declaration: &SingleTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let slang_declared_type = declaration.declaration().get_type();
        let declared_type = slang_declared_type
            .as_ref()
            .map(|slang_type| {
                slang_type.resolve_type(LocationPolicy::Declared(None), &self.state.builder)
            })
            .unwrap_or_else(|| {
                crate::ast::Type::unsigned(self.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir()
            });

        let emitter = ExpressionContext::from(&*self);

        // For explicit initializers, evaluate and cast before alloca to match
        // solc's emission order (constant → cast → alloca → store).
        // For implicit zero-initialization, alloca is emitted first.
        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
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
                    &self.state.builder,
                    &block,
                )
                .into_mlir();
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = if let Some(value) = initial_value {
            let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
            sol_op_void!(
                &self.state.builder,
                &block,
                StoreOperation.val(value).addr(pointer)
            );
            pointer
        } else {
            // No initializer: default-initialise the slot to the type's zero
            // through the shared primitive (memory aggregates malloc'd, empty
            // `string`/`bytes` a plain malloc, scalar value types their own
            // zero, integers a zeroed slot, references a bare slot).
            TypeConversion::emit_default_initialized_slot(
                slang_declared_type.as_ref(),
                declared_type,
                &self.state.builder,
                &block,
            )
        };

        self.environment.define_variable(
            declaration.declaration().node_id(),
            pointer,
            declared_type,
        );
        Ok(Some(block))
    }

    fn emit_multi_typed_declaration(
        &mut self,
        declaration: &MultiTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = declaration.value();
        let elements = declaration.elements();

        let emitter = ExpressionContext::from(&*self);

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
                let (values, current) =
                    emitter.emit_conditional_tuple_values(conditional, block)?;
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
            let builder = &self.state.builder;
            let declared_type = declaration
                .get_type()
                .map(|slang_type| slang_type.resolve_type(LocationPolicy::Declared(None), builder))
                .unwrap_or_else(|| {
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir()
                });
            let cast = crate::ast::Value::from(value)
                .coerce_to(crate::ast::Type::new(declared_type), builder, &current)
                .into_mlir();
            let pointer = builder.emit_sol_alloca(declared_type, &current);
            sol_op_void!(builder, &current, StoreOperation.val(cast).addr(pointer));
            self.environment
                .define_variable(declaration.node_id(), pointer, declared_type);
        }

        Ok(Some(current))
    }
}

statement_emit!(VariableDeclarationStatement; |node, context, block| {
    context.emit_variable_declaration(node, block)
});
