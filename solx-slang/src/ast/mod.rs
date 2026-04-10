//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;

use std::collections::BTreeMap;
use std::rc::Rc;

use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::backend::abi::AbiEntry;
use slang_solidity::backend::ir::ast::ContractMember;
use slang_solidity::backend::ir::ast::SourceUnit;

use solx_mlir::Context;

use self::contract::ContractEmitter;

/// Walks a Slang AST and lowers its contract definitions to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// Slang semantic analysis for resolving expression types.
    semantic: Rc<SemanticAnalysis>,
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(semantic: &Rc<SemanticAnalysis>, state: &'state mut Context<'context>) -> Self {
        Self {
            semantic: Rc::clone(semantic),
            state,
        }
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
        let file_identifier = unit.file_id();
        let mut emitter = ContractEmitter::new(&self.semantic, self.state);
        emitter.emit(contract, &file_identifier)?;

        let mut method_identifiers = BTreeMap::new();
        for contract_member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = contract_member else {
                continue;
            };
            let Some(AbiEntry::Function { name, inputs, .. }) = function.compute_abi_entry() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            // TODO: can be moved to slang-solidity
            let parameter_types: Vec<&str> =
                inputs.iter().map(|input| input.r#type.as_str()).collect();
            let signature = format!("{name}({})", parameter_types.join(","));
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok(Some((name, method_identifiers)))
    }
}
