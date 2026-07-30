//!
//! Source unit emission: lowering a file's contracts through the per-file MLIR scope.
//!

use std::collections::BTreeMap;

use itertools::Itertools;
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
    /// Lowers the unit's contracts, owning the per-file melior scope, into standard-JSON contract
    /// outputs keyed by contract name. Only the first contract per file compiles through this path
    /// today; the rest are skipped until inheritance-aware emission lands.
    ///
    /// # Errors
    ///
    /// Returns an error if module finalization fails.
    pub fn source_unit(
        unit: &SourceUnit,
        evm_version: EVMVersion,
        capture_sol_dialect: impl Fn(&str) -> bool,
    ) -> anyhow::Result<BTreeMap<String, Contract>> {
        let contracts = unit.contracts();
        let Some(contract) = contracts.first() else {
            return Ok(BTreeMap::new());
        };
        let melior = Context::create_melior_context();
        let mut scope = SourceUnitScope::new(Context::new(&melior, evm_version));
        let method_identifiers =
            scope.contract_definition(contract, &Self::operator_bound_functions(unit));

        let name = contract.name().name().to_owned();
        let mlir = Context::from(scope).finalize_module(
            &format!("{name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX),
            capture_sol_dialect(&name),
        )?;

        Ok(BTreeMap::from([(
            name,
            Contract::new_mlir(mlir, method_identifiers),
        )]))
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
