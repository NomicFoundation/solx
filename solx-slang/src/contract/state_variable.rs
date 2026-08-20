//!
//! State variable emission: the place a declaration resolves to and the inline initializers the
//! constructor runs.
//!

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_utils::DataLocation;

use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Emits every state variable's inline initializer (`T x = <expr>;`) in source order as the
    /// constructor prologue, storing each into its place. Reference-typed slots take a
    /// `sol.copy`; value-typed places convert to the declared element type and `sol.store`.
    pub fn state_variable_initializers(&mut self) {
        let initializers: Vec<(StateVariableDefinition, Expression)> = self
            .contract
            .state_variables
            .iter()
            .filter(|state_variable| {
                matches!(
                    state_variable.attributes().mutability(),
                    StateVariableMutability::Mutable | StateVariableMutability::Immutable
                )
            })
            .filter_map(|state_variable| Some((state_variable.clone(), state_variable.value()?)))
            .collect();
        for (state_variable, initializer) in initializers {
            let (storage_ref, element_type) = self.state_variable_place(&state_variable);
            if storage_ref.r#type() == element_type {
                storage_ref.copy_from(self.expression(&initializer), self);
            } else {
                storage_ref.store(self.converted(&initializer, element_type), self);
            }
        }
    }

    /// The `sol.addr_of` place of the state variable's storage slot or creation cell together with
    /// its element MLIR type, following the `Sol_GepOp` rule that a reference-typed element in
    /// storage is its own address.
    pub fn state_variable_place(
        &mut self,
        state_variable: &StateVariableDefinition,
    ) -> (Place<'context>, MlirType<'context>) {
        let declared_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        let element_type = self.resolve_type(&declared_type, None);
        (
            Place::addr_of(
                &SourceUnitScope::state_variable_symbol(state_variable),
                self.pointer_type(
                    &declared_type,
                    element_type,
                    DataLocation::from(state_variable.attributes().mutability()),
                ),
                self,
            ),
            element_type,
        )
    }
}

impl<'context> SourceUnitScope<'context> {
    /// The state variable's symbol: its name qualified by the node id, since names alone collide.
    pub fn state_variable_symbol(state_variable: &StateVariableDefinition) -> String {
        format!(
            "{}_{}",
            state_variable.name().name(),
            state_variable.node_id(),
        )
    }
}
