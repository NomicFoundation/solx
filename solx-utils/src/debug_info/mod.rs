//!
//! solc AST / debug info entities.
//!
//! Mostly consist of AST nodes and data.
//!

pub mod ast_node;
pub mod contract_definition;
pub mod function_definition;
pub mod line_index;
pub mod mapped_location;
pub mod solc_location;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
    /// Generic Solidity AST nodes, grouped by source ID.
    /// The outer key is the source ID; the inner key is the start byte offset.
    pub ast_nodes: HashMap<usize, HashMap<usize, AstNode>>,
    /// Source ID to source file path mapping.
    #[serde(default)]
    pub source_ids: BTreeMap<usize, String>,
}

impl DebugInfo {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        contract_definitions: HashMap<String, ContractDefinition>,
        function_definitions: HashMap<usize, FunctionDefinition>,
        ast_nodes: HashMap<usize, HashMap<usize, AstNode>>,
        source_ids: BTreeMap<usize, String>,
    ) -> Self {
        Self {
            contract_definitions,
            function_definitions,
            ast_nodes,
            source_ids,
        }
    }

    ///
    /// Builds the debug info restricted to the given source IDs and, when set, the current contract.
    /// Only the retained sources' AST nodes are cloned.
    ///
    pub fn filter_to(&self, source_ids: &BTreeSet<usize>, contract_name: Option<&str>) -> Self {
        Self {
            contract_definitions: self
                .contract_definitions
                .iter()
                .filter(|(name, contract_definition)| {
                    source_ids.contains(&contract_definition.solc_location.source_id)
                        && contract_name.is_none_or(|current| current == name.as_str())
                })
                .map(|(name, contract_definition)| (name.clone(), contract_definition.clone()))
                .collect(),
            function_definitions: self
                .function_definitions
                .iter()
                .filter(|(_, function_definition)| {
                    source_ids.contains(&function_definition.solc_location.source_id)
                })
                .map(|(id, function_definition)| (*id, function_definition.clone()))
                .collect(),
            ast_nodes: source_ids
                .iter()
                .filter_map(|source_id| {
                    self.ast_nodes
                        .get(source_id)
                        .map(|nodes| (*source_id, nodes.clone()))
                })
                .collect(),
            source_ids: self.source_ids.clone(),
        }
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
