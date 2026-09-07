//!
//! Source unit emission: lowering a file's contracts and libraries through the per-file MLIR
//! scope.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;

use solx_mlir::Context;
use solx_standard_json::output::contract::Contract;
use solx_utils::EVMVersion;

use crate::contract::object::Object;
use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Lowers every contract and library the unit deploys into standard-JSON contract outputs
    /// keyed by definition name, each in its own MLIR module off the file's melior context. An
    /// abstract contract and an interface deploy nothing and produce no module.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    pub fn source_unit(
        unit: &SourceUnit,
        evm_version: EVMVersion,
        capture_sol_dialect: impl Fn(&str) -> bool,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let melior = Context::create_melior_context();
        let mut contracts = BTreeMap::new();
        for member in unit.members().iter() {
            let object = match member {
                SourceUnitMember::ContractDefinition(contract) if !contract.is_abstract() => {
                    Object::Contract(contract.clone())
                }
                SourceUnitMember::LibraryDefinition(library) => Object::Library(library.clone()),
                _ => continue,
            };
            let name = object.name().name().to_owned();
            let identifier = object.identifier();
            let mut scope = SourceUnitScope::new(Context::new(&melior, evm_version));
            let method_identifiers = scope.object_definition(object);
            let mlir = Context::from(scope)
                .finalize_module(identifier.as_str(), capture_sol_dialect(name.as_str()))?;
            contracts.insert(name, Contract::new_mlir(mlir, method_identifiers));
        }
        Ok(contracts)
    }
}
