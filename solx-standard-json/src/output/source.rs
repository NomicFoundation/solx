//!
//! The `solc --standard-json` output source.
//!

use std::collections::BTreeMap;

use boolinator::Boolinator;

use crate::input::source::Source as StandardJsonInputSource;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Source code ID.
    pub id: usize,
    /// Source code AST.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<serde_json::Value>,
}

impl Source {
    ///
    /// Initializes a standard JSON source.
    ///
    /// Is used for projects compiled without `solc`.
    ///
    pub fn new(id: usize) -> Self {
        Self { id, ast: None }
    }

    ///
    /// Returns the list of messages for some specific parts of the AST.
    ///
    pub fn get_ast_nodes<K, V, F>(
        getter: &F,
        contract_name: &solx_utils::ContractName,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> BTreeMap<K, V>
    where
        K: std::cmp::Ord,
        V: solx_utils::IDebugInfoAstNode<Key = K>,
        F: Fn(
            &solx_utils::ContractName,
            &serde_json::Value,
            &BTreeMap<String, StandardJsonInputSource>,
        ) -> Option<V>,
    {
        let mut ast_nodes = BTreeMap::new();
        if let Some(ast_node) = getter(contract_name, ast, sources) {
            ast_nodes.insert(ast_node.index_id(), ast_node);
        }

        match ast {
            serde_json::Value::Array(array) => {
                for element in array.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, contract_name, element, sources));
                }
            }
            serde_json::Value::Object(object) => {
                for (_key, value) in object.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, contract_name, value, sources));
                }
            }
            _ => {}
        }

        ast_nodes
    }

    ///
    /// Returns a contract definition if the AST node is so.
    ///
    pub fn contract_definition(
        contract_name: &solx_utils::ContractName,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> Option<solx_utils::DebugInfoContractDefinition> {
        let ast = ast.as_object()?;

        (ast.get("nodeType")?.as_str()? == "ContractDefinition").as_option()?;

        let ast_id = ast.get("id")?.as_u64()? as usize;
        let name = ast.get("name")?.as_str()?.to_string();
        let solc_location = solx_utils::DebugInfoSolcLocation::parse(
            ast.get("src")?.as_str()?,
            solx_utils::DebugInfoSolcLocationOrdering::Ast,
        )
        .ok()?;
        let source_code = sources
            .get(contract_name.path.as_str())
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            contract_name,
            Some(solc_location.start),
            Some(solc_location.end),
            source_code,
        );

        Some(solx_utils::DebugInfoContractDefinition::new(
            ast_id,
            name,
            solc_location,
            mapped_location,
        ))
    }

    ///
    /// Returns a function definition if the AST node is so.
    ///
    pub fn function_definition(
        contract_name: &solx_utils::ContractName,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> Option<solx_utils::DebugInfoFunctionDefinition> {
        let ast = ast.as_object()?;

        (ast.get("nodeType")?.as_str()? == "FunctionDefinition").as_option()?;

        let ast_id = ast.get("id")?.as_u64()? as usize;
        let name = ast.get("name")?.as_str()?.to_owned();
        let solc_location = solx_utils::DebugInfoSolcLocation::parse(
            ast.get("src")?.as_str()?,
            solx_utils::DebugInfoSolcLocationOrdering::Ast,
        )
        .ok()?;
        let source_code = sources
            .get(contract_name.path.as_str())
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            contract_name,
            Some(solc_location.start),
            Some(solc_location.end),
            source_code,
        );

        Some(solx_utils::DebugInfoFunctionDefinition::new(
            ast_id,
            name,
            solc_location,
            mapped_location,
        ))
    }

    ///
    /// Returns an AST node if the JSON object is one.
    ///
    pub fn ast_node(
        contract_name: &solx_utils::ContractName,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> Option<solx_utils::DebugInfoAstNode> {
        let ast = ast.as_object()?;

        let ast_id = ast.get("id")?.as_u64()? as usize;
        let solc_location = solx_utils::DebugInfoSolcLocation::parse(
            ast.get("src")?.as_str()?,
            solx_utils::DebugInfoSolcLocationOrdering::Ast,
        )
        .ok()?;
        let source_code = sources
            .get(contract_name.path.as_str())
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            contract_name,
            Some(solc_location.start),
            Some(solc_location.end),
            source_code,
        );

        Some(solx_utils::DebugInfoAstNode::new(
            ast_id,
            solc_location,
            mapped_location,
        ))
    }
}
