//! Tuple deconstruction statement lowering.

use melior::ir::BlockRef;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::TupleDeconstructionMember;
use slang_solidity::backend::ir::ast::TupleDeconstructionStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a tuple deconstruction statement of the form
    /// `(decl_or_id_or_skip, ...) = (rhs0, rhs1, ...)`.
    ///
    /// The right-hand side must currently be a tuple expression; each item is
    /// emitted independently, then assigned to the corresponding LHS slot.
    /// `None` slots discard their value, `Identifier` slots store into an
    /// existing variable, and `VariableDeclarationStatement` slots allocate a
    /// new variable. Multi-result function calls on the RHS are not yet
    /// supported.
    pub fn emit_tuple_deconstruction(
        &mut self,
        statement: &TupleDeconstructionStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = statement.expression();
        let Expression::TupleExpression(tuple) = &expression else {
            anyhow::bail!(
                "tuple deconstruction with non-tuple right-hand side is not yet supported"
            );
        };

        let items = tuple.items();
        let members = statement.members();
        anyhow::ensure!(
            items.len() == members.len(),
            "tuple deconstruction arity mismatch: {} LHS slots vs {} RHS values",
            members.len(),
            items.len(),
        );

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let mut values = Vec::with_capacity(items.len());
        let mut current = block;
        for item in items.iter() {
            let inner = item
                .expression()
                .ok_or_else(|| anyhow::anyhow!("empty tuple element on RHS of deconstruction"))?;
            let (value, next) = emitter.emit_value(&inner, current)?;
            values.push(value);
            current = next;
        }

        for (member, value) in members.iter().zip(values.into_iter()) {
            match member {
                TupleDeconstructionMember::None => {
                    // Discard the value; nothing to bind.
                }
                TupleDeconstructionMember::Identifier(identifier) => {
                    let name = identifier.name();
                    let (pointer, target_type) = self
                        .environment
                        .variable_with_type(&name)
                        .ok_or_else(|| anyhow::anyhow!("unregistered local variable: {name}"))?;
                    let builder = &self.state.builder;
                    let cast = TypeConversion::from_target_type(target_type, builder)
                        .emit(value, builder, &current);
                    builder.emit_sol_store(cast, pointer, &current);
                }
                TupleDeconstructionMember::VariableDeclarationStatement(declaration) => {
                    let name = declaration.name().name();
                    let builder = &self.state.builder;
                    let declared_type = declaration
                        .get_type()
                        .map(|slang_type| {
                            TypeConversion::resolve_slang_type(&slang_type, None, builder)
                        })
                        .unwrap_or_else(|| builder.types.ui256);
                    let cast = TypeConversion::from_target_type(declared_type, builder)
                        .emit(value, builder, &current);
                    let pointer = builder.emit_sol_alloca(declared_type, &current);
                    builder.emit_sol_store(cast, pointer, &current);
                    self.environment
                        .define_variable(name, pointer, declared_type);
                }
            }
        }

        Ok(Some(current))
    }
}
