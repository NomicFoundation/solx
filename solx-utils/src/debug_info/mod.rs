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

use std::collections::HashMap;
use std::collections::HashSet;

use self::ast_node::AstNode;
use self::contract_definition::ContractDefinition;
use self::function_definition::FunctionDefinition;

///
/// Solidity debug info.
///
/// Each mapping has two keys:
///     1. Source code file ID.
///     2. See each mapping's description.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugInfo {
    /// Solidity AST contract definitions.
    /// The 2nd key is the contract name.
    pub contract_definitions: HashMap<usize, HashMap<String, ContractDefinition>>,
    /// Solidity AST function definitions.
    /// The 2nd key is the AST node ID.
    pub function_definitions: HashMap<usize, HashMap<usize, FunctionDefinition>>,
    /// Generic Solidity AST nodes.
    /// The 2nd key is the start byte offset.
    pub ast_nodes: HashMap<usize, HashMap<usize, AstNode>>,
}

impl DebugInfo {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        contract_definitions: HashMap<usize, HashMap<String, ContractDefinition>>,
        function_definitions: HashMap<usize, HashMap<usize, FunctionDefinition>>,
        ast_nodes: HashMap<usize, HashMap<usize, AstNode>>,
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
    pub fn retain_source_ids(&mut self, source_ids: &HashSet<usize>) {
        self.contract_definitions
            .retain(|source_id, _| source_ids.contains(source_id));
        self.function_definitions
            .retain(|source_id, _| source_ids.contains(source_id));
        self.ast_nodes
            .retain(|source_id, _| source_ids.contains(source_id));
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
