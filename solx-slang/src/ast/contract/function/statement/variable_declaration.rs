//! Tuple deconstruction statement lowering.

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiTypedDeclaration;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context> StatementEmitter<'state, 'context> {
    /// Emits a variable declaration with optional initializer.
    pub(super) fn emit_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        match declaration.target() {
            VariableDeclarationTarget::SingleTypedDeclaration(single_typed_declaration) => {
                self.emit_single_typed_declaration(&single_typed_declaration, context)
            }
            VariableDeclarationTarget::MultiTypedDeclaration(multi_typed_declaration) => {
                self.emit_multi_typed_declaration(&multi_typed_declaration, context)
            }
        }
    }

    fn emit_single_typed_declaration(
        &mut self,
        declaration: &SingleTypedDeclaration,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let name = declaration.declaration().name().name();
        let declared_type = declaration
            .declaration()
            .get_type()
            .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, context))
            .unwrap_or_else(|| Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD));

        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);

        let initial_value = if let Some(ref initializer_expression) = declaration.value() {
            let initial_value = emitter.emit_value(initializer_expression, context)?;
            let cast_value = TypeConversion::from_target_type(declared_type, context)
                .emit(initial_value, context);
            Some(cast_value)
        } else {
            None
        };

        let pointer = Place::stack(declared_type, context);

        if let Some(value) = initial_value {
            pointer.store(value, context);
        } else if declared_type.is_integer() {
            let zero = Value::constant(0, declared_type, context);
            pointer.store(zero, context);
        } else {
            unimplemented!("zero-initialization for non-integer type {declared_type}");
        }

        self.environment
            .define_variable(name, pointer, declared_type);
        Ok(())
    }

    fn emit_multi_typed_declaration(
        &mut self,
        declaration: &MultiTypedDeclaration,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let expression = declaration.value();
        let elements = declaration.elements();

        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);

        let values = match &expression {
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                anyhow::ensure!(
                    items.len() == elements.len(),
                    "tuple deconstruction arity mismatch: {} LHS slots vs {} RHS values",
                    elements.len(),
                    items.len(),
                );
                let mut values = Vec::with_capacity(items.len());
                for item in items.iter() {
                    let inner = item.expression().ok_or_else(|| {
                        anyhow::anyhow!("empty tuple element on RHS of deconstruction")
                    })?;
                    let value = emitter.emit_value(&inner, context)?;
                    values.push(value);
                }
                values
            }
            Expression::FunctionCallExpression(call) => {
                let call_emitter = CallEmitter::new(&emitter);
                let values = call_emitter.emit_function_call_results(call, context)?;
                anyhow::ensure!(
                    values.len() == elements.len(),
                    "tuple deconstruction arity mismatch: {} LHS slots vs {} call results",
                    elements.len(),
                    values.len(),
                );
                values
            }
            _ => anyhow::bail!(
                "tuple deconstruction with this right-hand side shape is not yet supported"
            ),
        };

        for (member, value) in elements.iter().zip(values) {
            let Some(declaration) = member.member() else {
                continue;
            };
            let name = declaration.name().name();
            let declared_type = declaration
                .get_type()
                .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, context))
                .unwrap_or_else(|| Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD));
            let cast =
                TypeConversion::from_target_type(declared_type, context).emit(value, context);
            let pointer = Place::stack(declared_type, context);
            pointer.store(cast, context);
            self.environment
                .define_variable(name, pointer, declared_type);
        }

        Ok(())
    }
}
