//! Variable declaration statement lowering.

use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
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
                TypeConversion::resolve_slang_type(slang_type, None, &self.state.builder)
            })
            .unwrap_or_else(|| self.state.builder.types.ui256);

        let emitter = self.expression_emitter();

        // For explicit initializers, evaluate and cast before alloca to match
        // solc's emission order (constant → cast → alloca → store).
        // For implicit zero-initialization, alloca is emitted first.
        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit_value(initializer_expression, block)?;
            let cast_value = TypeConversion::from_target_type(declared_type, &self.state.builder)
                .emit(initial_value, &self.state.builder, &block);
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = if let Some(value) = initial_value {
            let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
            self.state.builder.emit_sol_store(value, pointer, &block);
            pointer
        } else if matches!(
            slang_declared_type.as_ref(),
            Some(SlangType::FixedSizeArray(_) | SlangType::Struct(_))
        ) {
            // A memory aggregate (`T[n] memory a;`, `S memory s;`) declared
            // without an initializer is default-initialised by Solidity to a
            // freshly allocated zero-filled buffer, not left as a dangling
            // pointer (indexing it would otherwise revert / read garbage).
            let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
            let zero = self
                .state
                .builder
                .emit_sol_malloc_zeroed(declared_type, &block);
            self.state.builder.emit_sol_store(zero, pointer, &block);
            pointer
        } else if matches!(
            slang_declared_type.as_ref(),
            Some(SlangType::String(_) | SlangType::Bytes(_))
        ) {
            // A dynamic `string` / `bytes` without an initializer is an empty
            // (length-0) buffer, so `""` reads back rather than garbage.
            let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
            let size =
                self.state
                    .builder
                    .emit_sol_constant(0, self.state.builder.types.ui256, &block);
            let zero = self
                .state
                .builder
                .emit_sol_malloc_sized_zeroed(declared_type, size, &block);
            self.state.builder.emit_sol_store(zero, pointer, &block);
            pointer
        } else if let Some(
            scalar_value_type @ (SlangType::Address(_)
            | SlangType::ByteArray(_)
            | SlangType::Enum(_)
            | SlangType::UserDefinedValue(_)
            | SlangType::Function(_)),
        ) = slang_declared_type.as_ref()
        {
            // A value type that is not a plain integer/bool (an address,
            // `bytesN`, an enum, a UDVT over one, or a function pointer) needs
            // its representation's own zero, not a raw zeroed integer slot.
            let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
            let zero = TypeConversion::emit_scalar_zero(
                scalar_value_type,
                declared_type,
                &self.state.builder,
                &block,
            );
            self.state.builder.emit_sol_store(zero, pointer, &block);
            pointer
        } else {
            self.state
                .builder
                .emit_zero_initialized_alloca(declared_type, &block)
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

        let emitter = self.expression_emitter();

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
                    let (value, next) = emitter.emit_value(&inner, current)?;
                    values.push(value);
                    current = next;
                }
                (values, current)
            }
            Expression::FunctionCallExpression(call) => {
                let call_emitter = CallEmitter::new(&emitter);
                let (values, current) = call_emitter.emit_function_call_results(call, block)?;
                assert!(
                    values.len() == elements.len(),
                    "tuple deconstruction arity mismatch: {} LHS slots vs {} call results",
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
                .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, builder))
                .unwrap_or_else(|| builder.types.ui256);
            let cast = TypeConversion::from_target_type(declared_type, builder)
                .emit(value, builder, &current);
            let pointer = builder.emit_sol_alloca(declared_type, &current);
            builder.emit_sol_store(cast, pointer, &current);
            self.environment
                .define_variable(declaration.node_id(), pointer, declared_type);
        }

        Ok(Some(current))
    }
}
