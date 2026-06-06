//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;
pub mod expression_ext;
pub mod operator_binding;
/// Solidity type conversion classification and dispatch.
pub mod type_conversion;

pub use self::expression_ext::ExpressionExt;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::SourceUnit;

use solx_mlir::Context;

use self::contract::ContractEmitter;

/// Walks a Slang AST and lowers its contract definitions to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits MLIR for the first contract definition in the source unit.
    ///
    /// The current pipeline creates one MLIR module per source file, so
    /// only the first contract is processed. Multi-contract files will be
    /// supported in a future pass.
    ///
    /// Source files containing only interfaces, libraries, or abstract
    /// contracts are skipped without error.
    ///
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    /// Returns `Some((contract_name, method_identifiers))` if a contract was
    /// emitted, `None` otherwise.
    pub fn emit(
        &mut self,
        unit: &SourceUnit,
    ) -> anyhow::Result<Option<(String, BTreeMap<String, String>)>> {
        let contracts = unit.contracts();
        // TODO: support multiple contracts
        let Some(contract) = contracts.first() else {
            return Ok(None);
        };

        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract)?;

        let mut method_identifiers = BTreeMap::new();
        for contract_member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = contract_member else {
                continue;
            };
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok(Some((name, method_identifiers)))
    }
}
