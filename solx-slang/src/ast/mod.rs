//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;

use std::collections::BTreeMap;
use std::collections::HashMap;

use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::SourceUnitMember;

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

        // File-level functions are shared across the unit's contracts; gather
        // them so a contract that calls one emits it into its own module.
        let free_functions = Self::gather_free_functions(unit);

        // Map every external/public library function to its linker symbol, so a
        // `using`-for value-receiver delegatecall (`x.f(args)`) can resolve the
        // enclosing library — slang exposes no enclosing-library accessor on a
        // resolved function. Source-unit scoped, so set once here.
        self.state.library_function_symbols = Self::gather_library_function_symbols(unit);

        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract, &free_functions)?;

        let mut method_identifiers = BTreeMap::new();
        for contract_member in contract.members().iter() {
            match contract_member {
                ContractMember::FunctionDefinition(function) => {
                    let Some(signature) = function.compute_canonical_signature() else {
                        continue;
                    };
                    let Some(selector) = function.compute_selector() else {
                        continue;
                    };
                    method_identifiers.insert(signature, format!("{selector:08x}"));
                }
                ContractMember::StateVariableDefinition(state_variable) => {
                    let Some(signature) = state_variable.compute_canonical_signature() else {
                        continue;
                    };
                    let Some(selector) = state_variable.compute_selector() else {
                        continue;
                    };
                    method_identifiers.insert(signature, format!("{selector:08x}"));
                }
                _ => {}
            }
        }

        Ok(Some((name, method_identifiers)))
    }

    /// Collects the source unit's file-level (free) function definitions.
    fn gather_free_functions(unit: &SourceUnit) -> Vec<FunctionDefinition> {
        unit.members()
            .iter()
            .filter_map(|member| match member {
                SourceUnitMember::FunctionDefinition(function) => Some(function),
                _ => None,
            })
            .collect()
    }

    /// Maps every `external`/`public` library function's definition id to its
    /// enclosing library's linker symbol (`file_id:LibraryName`), for the
    /// `using`-for value-receiver delegatecall path.
    fn gather_library_function_symbols(unit: &SourceUnit) -> HashMap<NodeId, String> {
        let mut symbols = HashMap::new();
        for member in unit.members().iter() {
            let SourceUnitMember::LibraryDefinition(library) = member else {
                continue;
            };
            let symbol = format!("{}:{}", library.get_file_id(), library.name().name());
            for library_member in library.members().iter() {
                if let ContractMember::FunctionDefinition(function) = library_member
                    && matches!(
                        function.visibility(),
                        FunctionVisibility::External | FunctionVisibility::Public
                    )
                {
                    symbols.insert(function.node_id(), symbol.clone());
                }
            }
        }
        symbols
    }
}
