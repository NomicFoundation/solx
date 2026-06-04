//!
//! Identifier expression lowering: reads of locals and parameters.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ConstantDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an identifier reference to the value it denotes.
    ///
    /// Locals and parameters are stack slots: the binding is looked up in the
    /// environment and loaded. Other binding kinds (state variables, constants,
    /// functions, libraries) are lowered by their own domains.
    pub fn emit_identifier(
        &self,
        identifier: &Identifier,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let name = identifier.name();
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let value = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, &block)?;
                Ok((value, block))
            }
            Some(Definition::StateVariable(state_variable)) => {
                self.emit_state_variable_read(&state_variable, block)
            }
            Some(Definition::Constant(constant)) => self.emit_constant_read(&constant, block),
            Some(_) => unimplemented!("identifier reference lowering: {name}"),
            None => unreachable!("unresolved identifier: {name}"),
        }
    }

    /// Reads a `constant`: its initializer expression is inlined at the use
    /// site and cast to the constant's declared type. An initializer that
    /// references another constant inlines recursively.
    fn emit_constant_read(
        &self,
        constant: &ConstantDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let initializer = constant.value().expect("a constant has an initializer");
        let element_type = TypeConversion::resolve_slang_type(
            &constant
                .get_type()
                .expect("the binder types every constant"),
            None,
            &self.state.builder,
        );
        let (value, block) = self.emit_value(&initializer, block)?;
        let value = TypeConversion::from_target_type(element_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        Ok((value, block))
    }
}
