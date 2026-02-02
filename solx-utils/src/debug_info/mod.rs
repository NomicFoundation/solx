//!
//! solc AST / debug info entities.
//!
//! Mostly consist of AST nodes and data.
//!

pub mod ast_node;
pub mod contract_definition;
pub mod function_definition;
pub mod mapped_location;
pub mod solc_location;

use std::collections::BTreeMap;
use std::collections::HashMap;

use self::ast_node::AstNode;
use self::contract_definition::ContractDefinition;
use self::function_definition::FunctionDefinition;

///
/// Solidity debug info.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugInfo {
    /// Solidity AST contract definition of the current contract.
    /// The key is the contract full path.
    pub contract_definitions: HashMap<String, ContractDefinition>,
    /// Solidity AST function definitions.
    /// The key is the AST node ID.
    pub function_definitions: HashMap<usize, FunctionDefinition>,
    /// Generic Solidity AST nodes.
    /// The key is the start byte offset.
    pub ast_nodes: HashMap<usize, AstNode>,
}

impl DebugInfo {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        contract_definitions: HashMap<String, ContractDefinition>,
        function_definitions: HashMap<usize, FunctionDefinition>,
        ast_nodes: HashMap<usize, AstNode>,
    ) -> Self {
        Self {
            contract_definitions,
            function_definitions,
            ast_nodes,
        }
    }

    ///
    /// Retains only the debug info for the specified source IDs.
    ///
    pub fn retain_source_ids(&mut self, source_ids: &BTreeMap<usize, String>) {
        self.contract_definitions.retain(|_, contract_definition| {
            source_ids.contains_key(&contract_definition.solc_location.source_id)
        });
        self.function_definitions.retain(|_, function_definition| {
            source_ids.contains_key(&function_definition.solc_location.source_id)
        });
        self.ast_nodes
            .retain(|_, ast_node| source_ids.contains_key(&ast_node.solc_location.source_id));
    }

    ///
    /// Retains only the contract definition of the current contract.
    ///
    pub fn retain_current_contract(&mut self, contract_name: &str) {
        self.contract_definitions
            .retain(|name, _| name == contract_name);
    }
}

///
/// AST node trait.
///
pub trait IDebugInfoAstNode {
    /// The AST node key type.
    type Key;

    ///
    /// Returns the identifier used for looking up the node in debug info mappings.
    ///
    fn index_id(&self) -> Self::Key;
}
