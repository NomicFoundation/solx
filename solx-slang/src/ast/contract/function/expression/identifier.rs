//!
//! Identifier expression lowering: reads of locals, parameters, state
//! variables, and constants.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an identifier reference to the value it denotes.
    pub fn emit_identifier(
        &self,
        identifier: &Identifier,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let name = identifier.name();
        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("every state variable has a storage slot");
                let declared_type = state_variable
                    .get_type()
                    .expect("the binder types every state variable");
                let element_type =
                    TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
                let address = self.state.builder.emit_sol_addr_of(
                    &slot.name,
                    Self::address_type(
                        &self.state.builder,
                        element_type,
                        DataLocation::Storage,
                        &declared_type,
                    ),
                    &block,
                );
                let value = self
                    .state
                    .builder
                    .emit_sol_load(address, element_type, &block)?;
                Ok((value, block))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let value = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, &block)?;
                Ok((value, block))
            }
            Some(Definition::Constant(constant)) => {
                let initializer = constant.value().expect("a constant has an initializer");
                self.emit_value(&initializer, block)
            }
            None => unreachable!("slang resolves every identifier reference: {name}"),
            Some(_) => unimplemented!("reference to '{name}' is not yet supported"),
        }
    }
}
