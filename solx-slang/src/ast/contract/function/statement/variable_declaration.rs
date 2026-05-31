//! Tuple deconstruction statement lowering.

use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a variable declaration with optional initializer.
    pub(super) fn emit_variable_declaration(
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
        let name = declaration.declaration().name().name();
        let slang_declared_type = declaration.declaration().get_type();
        let declared_type = slang_declared_type
            .as_ref()
            .map(|slang_type| {
                TypeConversion::resolve_slang_type(slang_type, None, &self.state.builder)
            })
            .unwrap_or_else(|| self.state.builder.types.ui256);
        // A value-type memory aggregate declared without an initializer
        // (`T[n] memory a;`, `S memory s;`) is zero-allocated in memory by
        // Solidity; without that, indexing it reverts. Allocate fixed-size
        // arrays and structs up front.
        let needs_memory_alloc = declaration.value().is_none()
            && matches!(
                slang_declared_type,
                Some(
                    slang_solidity_v2::ast::Type::FixedSizeArray(_)
                        | slang_solidity_v2::ast::Type::Struct(_)
                )
            );

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        // For explicit initializers, evaluate and cast before alloca to match
        // solc's emission order (constant → cast → alloca → store).
        // For implicit zero-initialization, alloca is emitted first.
        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit_value(initializer_expression, block)?;
            let cast_value = TypeConversion::from_target_type(
                declared_type,
                &emitter.state.builder,
            )
            .emit(initial_value, &emitter.state.builder, &block);
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = emitter.state.builder.emit_sol_alloca(declared_type, &block);

        if let Some(value) = initial_value {
            emitter.state.builder.emit_sol_store(value, pointer, &block);
        } else if melior::ir::r#type::IntegerType::try_from(declared_type).is_ok() {
            let zero = self
                .state
                .builder
                .emit_sol_constant(0, declared_type, &block);
            emitter.state.builder.emit_sol_store(zero, pointer, &block);
        } else if needs_memory_alloc {
            // Allocate a fresh zero-initialised aggregate in memory and bind
            // the variable to it.
            let allocated = emitter.state.builder.emit_sol_malloc(declared_type, &block);
            emitter.state.builder.emit_sol_store(allocated, pointer, &block);
        }
        // Other non-integer declarations without an initializer are left as
        // the raw sol.alloca pointer.

        self.environment
            .define_variable(name, pointer, declared_type);
        Ok(Some(block))
    }

    fn emit_multi_typed_declaration(
        &mut self,
        declaration: &MultiTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = declaration.value();
        let elements = declaration.elements();

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let (values, current) = match &expression {
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                anyhow::ensure!(
                    items.len() == elements.len(),
                    "tuple deconstruction arity mismatch: {} LHS slots vs {} RHS values",
                    elements.len(),
                    items.len(),
                );
                let mut values = Vec::with_capacity(items.len());
                let mut current = block;
                for item in items.iter() {
                    let inner = item.expression().ok_or_else(|| {
                        anyhow::anyhow!("empty tuple element on RHS of deconstruction")
                    })?;
                    let (value, next) = emitter.emit_value(&inner, current)?;
                    values.push(value);
                    current = next;
                }
                (values, current)
            }
            Expression::FunctionCallExpression(call) => {
                let call_emitter = CallEmitter::new(&emitter);
                let (values, current) = call_emitter.emit_function_call_results(call, block)?;
                anyhow::ensure!(
                    values.len() == elements.len(),
                    "tuple deconstruction arity mismatch: {} LHS slots vs {} call results",
                    elements.len(),
                    values.len(),
                );
                (values, current)
            }
            _ => anyhow::bail!(
                "tuple deconstruction with this right-hand side shape is not yet supported"
            ),
        };

        for (member, value) in elements.iter().zip(values) {
            let Some(declaration) = member.member() else {
                // Discard the value; nothing to bind.
                continue;
            };
            let name = declaration.name().name();
            let builder = &self.state.builder;
            let declared_type = declaration
                .get_type()
                .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, builder))
                .unwrap_or_else(|| builder.types.ui256);
            let cast = TypeConversion::from_target_type(declared_type, builder)
                .emit(value, builder, &current);
            let pointer = builder.emit_sol_alloca(declared_type, &current);
            builder.emit_sol_store(cast, pointer, &current);
            self.environment
                .define_variable(name, pointer, declared_type);
        }

        Ok(Some(current))
    }
}
