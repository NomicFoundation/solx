//!
//! Source unit emission: lowering a file's contracts and libraries through the per-file MLIR
//! scope.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractBase;
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
    /// abstract contract and an interface deploy nothing and produce no module. A contract with a
    /// contract base is skipped because emission collects only the contract's own state and
    /// functions: an interface base carries nothing to inherit, a contract base carries state and
    /// bodies that would be silently dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    // TODO: emit a contract inheriting from a contract, which needs its inherited state variables
    // and functions.
    pub fn source_unit(
        unit: &SourceUnit,
        evm_version: EVMVersion,
        capture_sol_dialect: impl Fn(&str) -> bool,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let melior = Context::create_melior_context();
        let mut contracts = BTreeMap::new();
        for member in unit.members().iter() {
            let object = match member {
                SourceUnitMember::ContractDefinition(contract)
                    if !contract.is_abstract()
                        && !contract.direct_bases().iter().any(|base| match base {
                            ContractBase::Contract(_) => true,
                            ContractBase::Interface(_) => false,
                        }) =>
                {
                    Object::Contract(contract.clone())
                }
                SourceUnitMember::LibraryDefinition(library) => Object::Library(library.clone()),
                _ => continue,
            };
            let mut scope = SourceUnitScope::new(Context::new(&melior, evm_version));
            let method_identifiers = scope.object_definition(&object);
            let name = object.name().name().to_owned();
            let mlir = Context::from(scope).finalize_module(
                object.identifier().as_str(),
                capture_sol_dialect(name.as_str()),
            )?;
            contracts.insert(name, Contract::new_mlir(mlir, method_identifiers));
        }
        Ok(contracts)
    }
}
