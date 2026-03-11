//!
//! Source unit (top-level file) lowering to MLIR.
//!

use slang_solidity::backend::ir::ir2_flat_contracts::SourceUnit;
use slang_solidity::backend::ir::ir2_flat_contracts::SourceUnitMember;

use crate::codegen::MlirContext;
use crate::codegen::contract::ContractEmitter;

/// Walks a `SourceUnit` and lowers its contract definitions to MLIR.
pub struct SourceUnitEmitter<'state, 'context> {
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
    pub(crate) fn emit(&mut self, unit: &SourceUnit) -> anyhow::Result<bool> {
        for member in &unit.members {
            if let SourceUnitMember::ContractDefinition(contract) = member {
                let mut emitter = ContractEmitter::new(self.state);
                emitter.emit(contract)?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}
