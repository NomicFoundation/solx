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

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_statement::EmitStatement;
use crate::ast::emit::emit_values::EmitValues;

statement_emit!(VariableDeclarationStatement; |node, context, block| {
    match node.target() {
        VariableDeclarationTarget::SingleTypedDeclaration(single_typed_declaration) => {
            context.emit_single_typed_declaration(&single_typed_declaration, block)
        }
        VariableDeclarationTarget::MultiTypedDeclaration(multi_typed_declaration) => {
            context.emit_multi_typed_declaration(&multi_typed_declaration, block)
        }
    }
});

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a single-typed variable declaration with an optional initializer.
    fn emit_single_typed_declaration(
        &mut self,
        declaration: &SingleTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let name = declaration.declaration().name().name();
        let declared_type = declaration
            .declaration()
            .get_type()
            .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, self.state))
            .unwrap_or_else(|| {
                AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            });

        let expression_context = self.expression_context();
        let (block, initial_value) = if let Some(ref initializer_expression) = declaration.value() {
            let (block, cast_value) = match initializer_expression {
                Expression::StringExpression(string_literal)
                    if expression_context
                        .fixed_bytes_or_byte_width(declared_type)
                        .is_some() =>
                {
                    let BlockAnd { value, block } =
                        string_literal.emit_as(declared_type, &expression_context, block);
                    (block, value)
                }
                _ => {
                    let BlockAnd {
                        value: initial_value,
                        block,
                    } = initializer_expression.emit(&expression_context, block);
                    let cast_value = TypeConversion::from_target_type(declared_type, self.state)
                        .emit(initial_value, self.state, &block);
                    (block, cast_value)
                }
            };
            (block, Some(cast_value))
        } else {
            (block, None)
        };

        let pointer = Pointer::stack(AstType::new(declared_type), self.state, &block);

        if let Some(value) = initial_value {
            pointer.store(AstValue::new(value), self.state, &block);
        } else if melior::ir::r#type::IntegerType::try_from(declared_type).is_ok() {
            let zero = AstValue::constant(0, AstType::new(declared_type), self.state, &block);
            pointer.store(zero, self.state, &block);
        } else {
            unimplemented!("zero-initialization for non-integer type {declared_type}");
        }

        let pointer = pointer.into_mlir();

        self.environment.define_variable(name, pointer, declared_type);
        Some(block)
    }

    /// Emits a multi-typed variable declaration, deconstructing a tuple or call.
    fn emit_multi_typed_declaration(
        &mut self,
        declaration: &MultiTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let expression = declaration.value();
        let elements = declaration.elements();

        let expression_context = self.expression_context();
        let BlockAnd {
            value: values,
            block: current,
        } = match &expression {
            Expression::TupleExpression(_) | Expression::FunctionCallExpression(_) => {
                expression.emit_values(&expression_context, block)
            }
            _ => unimplemented!(
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
                .map(|slang_type| TypeConversion::resolve_slang_type(&slang_type, None, self.state))
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

        Some(current)
    }
}
