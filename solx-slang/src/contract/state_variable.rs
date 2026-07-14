//!
//! State variable definition emission: the storage place a declaration resolves to.
//!

use slang_solidity_v2::ast::StateVariableDefinition as SlangStateVariableDefinition;

use solx_mlir::Context as MlirContext;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_utils::DataLocation;

use crate::r#type::Type;

codegen!(
    StateVariableDefinition {
        /// The `sol.addr_of` place of the variable's storage slot together with its element MLIR
        /// type, following the `Sol_GepOp` rule that a reference-typed element in storage is its
        /// own address.
        pub fn storage_place<'context>(
            state_variable: &SlangStateVariableDefinition,
            slot_name: &str,
            context: &MlirContext<'context>,
        ) -> (Place<'context>, MlirType<'context>) {
            let declared_type = state_variable
                .get_type()
                .expect("binder types every state variable");
            let element_type = Type::resolve(&declared_type, None, context);
            let address_type =
                Type::address_type(&declared_type, element_type, DataLocation::Storage, context);
            (
                Place::addr_of(slot_name, address_type, context),
                element_type,
            )
        }
    }
);
