//!
//! Source unit emission: lowering a file's contracts through the per-file MLIR scope.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::SourceUnit;

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
        let method_identifiers = scope.contract_definition(contract);

        let name = contract.name().name();
        let mlir = Context::from(scope).finalize_module(
            &format!("{name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX),
            capture_sol_dialect(&name),
        )?;

        Ok(BTreeMap::from([(
            name,
            Contract::new_mlir(mlir, method_identifiers),
        )]))
    }
}
