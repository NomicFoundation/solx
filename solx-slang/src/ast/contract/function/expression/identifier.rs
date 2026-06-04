//!
//! Identifier expression lowering: reads of locals and parameters.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;

use crate::ast::contract::function::expression::ExpressionEmitter;

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
            Some(_) => unimplemented!("identifier reference lowering: {name}"),
            None => unreachable!("unresolved identifier: {name}"),
        }
    }
}
