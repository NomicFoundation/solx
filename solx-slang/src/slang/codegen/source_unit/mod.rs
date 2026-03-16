//!
//! Source unit (top-level file) lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub(crate) mod contract;

use slang_solidity::backend::ir::ast::SourceUnit;
use slang_solidity::backend::ir::ast::SourceUnitMember;

use crate::slang::codegen::MlirContext;

use self::contract::ContractEmitter;

/// Walks a `SourceUnit` and lowers its contract definitions to MLIR.
pub(crate) struct SourceUnitEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut MlirContext<'context>,
}

impl<'state, 'context> SourceUnitEmitter<'state, 'context> {
    /// Creates a new source unit emitter.
    pub(crate) fn new(state: &'state mut MlirContext<'context>) -> Self {
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
    /// Returns `Some(contract_name)` if a contract was emitted, `None` otherwise.
    pub(crate) fn emit(&mut self, unit: &SourceUnit) -> anyhow::Result<Option<String>> {
        for member in unit.members().iter() {
            if let SourceUnitMember::ContractDefinition(contract) = member {
                let name = contract.name().name();
                let mut emitter = ContractEmitter::new(self.state);
                emitter.emit(&contract)?;
                return Ok(Some(name));
            }
        }

        Ok(None)
    }
}
