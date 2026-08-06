//!
//! Source unit emission: lowering a file's contracts through the per-file MLIR scope.
//!

use std::collections::BTreeMap;

use itertools::Itertools;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::ast::UsingClause;

use solx_mlir::Context;
use solx_standard_json::output::contract::Contract;
use solx_utils::EVMVersion;

use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Lowers every contract the unit deploys into standard-JSON contract outputs keyed by contract
    /// name, each in its own MLIR module off the file's melior context. An abstract contract and an
    /// interface deploy nothing and produce no module. A contract with a contract base is skipped
    /// because emission collects only the contract's own state and functions: an interface base
    /// carries nothing to inherit, a contract base carries state and bodies that would be silently
    /// dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    // TODO: emit a contract inheriting from a contract, which needs its inherited state variables
    // and functions, and a library, whose own address reaches LLVM through the untranslated
    // `llvm.setimmutable`.
    pub fn source_unit(
        unit: &SourceUnit,
        evm_version: EVMVersion,
        capture_sol_dialect: impl Fn(&str) -> bool,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let operator_functions = Self::operator_bound_functions(unit);
        let melior = Context::create_melior_context();
        let mut contracts = BTreeMap::new();
        for contract in unit.contracts().iter().filter(|contract| {
            !contract.is_abstract()
                && !contract.direct_bases().iter().any(|base| match base {
                    ContractBase::Contract(_) => true,
                    ContractBase::Interface(_) => false,
                })
        }) {
            let mut scope = SourceUnitScope::new(Context::new(&melior, evm_version));
            let method_identifiers = scope.contract_definition(contract, &operator_functions);

            let name = contract.name().name().to_owned();
            let object_identifier = Self::object_identifier(contract.get_file_id(), name.as_str());
            let mlir = Context::from(scope).finalize_module(
                object_identifier.as_str(),
                capture_sol_dialect(name.as_str()),
            )?;
            contracts.insert(name, Contract::new_mlir(mlir, method_identifiers));
        }
        Ok(contracts)
    }

    /// The functions `unit`'s `using {f as op} for T global;` directives bind operators to, in
    /// source order and without repeats. A binding reaches the whole compilation unit, so a
    /// function bound from another file is missing here.
    // TODO: add support for cross-file binding of operator functions
    fn operator_bound_functions(unit: &SourceUnit) -> Vec<FunctionDefinition> {
        unit.members()
            .iter()
            .filter_map(|member| match member {
                SourceUnitMember::UsingDirective(directive) if directive.is_global() => {
                    match directive.clause() {
                        UsingClause::UsingDeconstruction(deconstruction) => Some(deconstruction),
                        UsingClause::IdentifierPath(_) => None,
                    }
                }
                _ => None,
            })
            .flat_map(|deconstruction| deconstruction.symbols().iter().collect::<Vec<_>>())
            .filter(|symbol| symbol.alias().is_some())
            .filter_map(|symbol| match symbol.name().resolve_to_definition()? {
                Definition::Function(function) => Some(function),
                _ => None,
            })
            .unique_by(|function| function.node_id())
            .collect()
    }
}
