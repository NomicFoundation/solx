//! Tuple deconstruction statement lowering.

use melior::ir::BlockRef;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

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
        let declared_type = declaration
            .declaration()
            .get_type()
            .map(|slang_type| {
                TypeConversion::resolve_slang_type(&slang_type, None, self.state)
            })
            .unwrap_or_else(|| {
                AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            });

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
            let (initial_value, block) = emitter.emit_value(initializer_expression, block)?;
            let cast_value = TypeConversion::from_target_type(declared_type, emitter.state)
                .emit(initial_value, emitter.state, &block);
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = Pointer::stack(AstType::new(declared_type), emitter.state, &block);

        if let Some(value) = initial_value {
            pointer.store(AstValue::new(value), emitter.state, &block);
        } else if melior::ir::r#type::IntegerType::try_from(declared_type).is_ok() {
            let zero = AstValue::constant(0, AstType::new(declared_type), self.state, &block);
            pointer.store(zero, emitter.state, &block);
        } else {
            unimplemented!("zero-initialization for non-integer type {declared_type}");
        }

        let pointer = pointer.into_mlir();

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
            let declared_type = declaration
                .get_type()
                .map(|slang_type| {
                    TypeConversion::resolve_slang_type(&slang_type, None, self.state)
                })
                .unwrap_or_else(|| {
                    AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir()
                });
            let cast = TypeConversion::from_target_type(declared_type, self.state)
                .emit(value, self.state, &current);
            let pointer = Pointer::stack(AstType::new(declared_type), self.state, &current);
            pointer.store(AstValue::new(cast), self.state, &current);
            self.environment
                .define_variable(name, pointer.into_mlir(), declared_type);
        }

        Ok(Some(current))
    }
}
