//!
//! Local variable declaration statement lowering.
//!

use melior::ir::BlockRef;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::SingleTypedDeclaration;
use slang_solidity_v2::ast::VariableDeclarationStatement;
use slang_solidity_v2::ast::VariableDeclarationTarget;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

use super::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a local variable declaration.
    pub(super) fn emit_variable_declaration(
        &mut self,
        declaration: &VariableDeclarationStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match declaration.target() {
            VariableDeclarationTarget::SingleTypedDeclaration(single) => {
                self.emit_single_typed_declaration(&single, block)
            }
            VariableDeclarationTarget::MultiTypedDeclaration(_) => {
                unimplemented!("tuple deconstruction declaration lowering")
            }
        }
    }

    /// Lowers a single `T name [= value];` declaration: allocate a stack slot,
    /// store the initializer (or a typed zero when omitted), and bind the name.
    fn emit_single_typed_declaration(
        &mut self,
        declaration: &SingleTypedDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let name = declaration.declaration().name().name();
        let declared_slang_type = declaration
            .declaration()
            .get_type()
            .expect("binder types every variable declaration");
        let declared_type =
            TypeConversion::resolve_slang_type(&declared_slang_type, None, &self.state.builder);

        // An explicit initializer is evaluated and cast before the alloca, to
        // match solc's emission order (value → alloca → store).
        let (block, initial_value) = match declaration.value() {
            Some(initializer) => {
                let emitter = ExpressionEmitter::new(
                    self.state,
                    self.environment,
                    self.storage_layout,
                    self.checked,
                );
                let (value, block) = emitter.emit_value(&initializer, block)?;
                let value = TypeConversion::from_target_type(declared_type, &self.state.builder)
                    .emit(value, &self.state.builder, &block);
                (block, Some(value))
            }
            None => (block, None),
        };

        let pointer = self.state.builder.emit_sol_alloca(declared_type, &block);
        let value = match initial_value {
            Some(value) => value,
            None if IntegerType::try_from(declared_type).is_ok() => self
                .state
                .builder
                .emit_sol_constant(0, declared_type, &block),
            None => unimplemented!("zero-initialization for non-integer type {declared_type}"),
        };
        self.state.builder.emit_sol_store(value, pointer, &block);
        self.environment
            .define_variable(name, pointer, declared_type);
        Ok(Some(block))
    }
}
