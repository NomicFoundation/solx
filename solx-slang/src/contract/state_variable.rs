//!
//! State variable emission: the storage place a declaration resolves to and the inline
//! initializers the constructor runs.
//!

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Place;
use solx_mlir::Type;
use solx_utils::DataLocation;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits every state variable's inline initializer (`T x = <expr>;`) in source order as the
    /// constructor prologue, storing each into its storage slot. Reference-typed slots take a
    /// `sol.copy`; value-typed slots coerce to the declared element type and `sol.store`.
    pub fn state_variable_initializers(&mut self) {
        let initializers: Vec<(StateVariableDefinition, String, Expression)> = self
            .contract
            .state_variables
            .iter()
            .filter_map(|state_variable| {
                Some((
                    state_variable.clone(),
                    self.contract
                        .storage_layout
                        .get(&state_variable.node_id())?
                        .name
                        .clone(),
                    state_variable.value()?,
                ))
            })
            .collect();
        for (state_variable, slot_name, initializer) in initializers {
            if matches!(initializer, Expression::ArrayExpression(_)) {
                unimplemented!("array-literal state variable initializers are not yet supported");
            }
            let (storage_ref, element_type) =
                self.state_variable_place(&state_variable, &slot_name);
            let value = self.expression(&initializer);
            if storage_ref.r#type() == element_type {
                storage_ref.copy_from(value, self);
            } else {
                storage_ref.store(value.coerce(element_type, self), self);
            }
        }
    }

    /// The `sol.addr_of` place of the state variable's storage slot together with its element MLIR
    /// type, following the `Sol_GepOp` rule that a reference-typed element in storage is its own
    /// address.
    pub fn state_variable_place(
        &mut self,
        state_variable: &StateVariableDefinition,
        slot_name: &str,
    ) -> (Place<'context>, Type<'context>) {
        let declared_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        let element_type = self.resolve_type(&declared_type, None);
        (
            Place::addr_of(
                slot_name,
                self.pointer_type(&declared_type, element_type, DataLocation::Storage),
                self,
            ),
            element_type,
        )
    }
}
