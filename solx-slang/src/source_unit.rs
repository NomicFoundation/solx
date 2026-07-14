//!
//! Source unit emission: the per-file MLIR scope and its contracts' lowering artifacts.
//!

use std::collections::BTreeMap;

use crate::contract::ContractDefinition;
use crate::scope::SourceUnitScope;

/// One contract's lowering artifacts, keyed for standard-JSON assembly.
pub struct EmittedContract {
    /// The contract name.
    pub name: String,
    /// The ABI `method_identifiers` map.
    pub method_identifiers: BTreeMap<String, String>,
    /// The finalized MLIR stages.
    pub mlir: solx_mlir::MlirOutput,
}

codegen!(
    SourceUnit {
        /// Lowers the unit's contracts, owning the per-file melior scope. Only the first contract
        /// per file compiles through this path today; the rest are skipped until
        /// inheritance-aware emission lands.
        ///
        /// # Errors
        ///
        /// Returns an error if module finalization fails.
        pub fn emit_contracts(
            unit: &slang_solidity_v2::ast::SourceUnit,
            evm_version: solx_utils::EVMVersion,
            capture_sol_dialect: impl Fn(&str) -> bool,
        ) -> anyhow::Result<Vec<EmittedContract>> {
            let contracts = unit.contracts();
            let Some(contract) = contracts.first() else {
                return Ok(Vec::new());
            };
            let melior = solx_mlir::Context::create_melior_context();
            let mut scope = SourceUnitScope::new(solx_mlir::Context::new(&melior, evm_version));
            ContractDefinition::emit(contract, &mut scope);

            let name = contract.name().name();
            let runtime_code_identifier =
                format!("{name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX);
            let mlir = solx_mlir::Context::from(scope)
                .finalize_module(&runtime_code_identifier, capture_sol_dialect(&name))?;

            Ok(vec![EmittedContract {
                method_identifiers: ContractDefinition::method_identifiers(contract),
                name,
                mlir,
            }])
        }
    }
);
