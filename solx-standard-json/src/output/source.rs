//!
//! The `solc --standard-json` output source.
//!

use std::collections::BTreeMap;

use boolinator::Boolinator;

use crate::input::source::Source as StandardJsonInputSource;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Returns the name of the last contract in the AST.
    ///
    pub fn last_contract_name(&self) -> anyhow::Result<String> {
        self.ast
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The AST is empty"))?
            .get("nodes")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("The last contract cannot be found in an empty list of nodes")
            })?
            .iter()
            .filter_map(
                |node| match node.get("nodeType").and_then(|node| node.as_str()) {
                    Some("ContractDefinition") => Some(node.get("name")?.as_str()?.to_owned()),
                    _ => None,
                },
            )
            .next_back()
            .ok_or_else(|| anyhow::anyhow!("The last contract not found in the AST"))
    }

    ///
    /// Returns the list of messages for some specific parts of the AST.
    ///
    pub fn get_ast_nodes<K, V, F>(
        getter: &F,
        path: &str,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> BTreeMap<K, V>
    where
        K: std::cmp::Ord,
        V: solx_utils::IDebugInfoAstNode<Key = K>,
        F: Fn(&str, &serde_json::Value, &BTreeMap<String, StandardJsonInputSource>) -> Option<V>,
    {
        let mut ast_nodes = BTreeMap::new();
        if let Some(ast_node) = getter(path, ast, sources) {
            ast_nodes.insert(ast_node.index_id(), ast_node);
        }

        match ast {
            serde_json::Value::Array(array) => {
                for element in array.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, path, element, sources));
                }
            }
            serde_json::Value::Object(object) => {
                for (_key, value) in object.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, path, value, sources));
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
        path: &str,
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
            .get(path)
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            path.to_owned(),
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
        path: &str,
        ast: &serde_json::Value,
        sources: &BTreeMap<String, StandardJsonInputSource>,
    ) -> Option<solx_utils::DebugInfoFunctionDefinition> {
        let ast = ast.as_object()?;

        let is_function = ast.get("nodeType")?.as_str()? == "FunctionDefinition";
        let is_modifier = ast.get("nodeType")?.as_str()? == "ModifierDefinition";
        let is_storage_variable = (ast.get("nodeType")?.as_str()? == "VariableDeclaration")
            && ast.contains_key("functionSelector");
        (is_function || is_modifier || is_storage_variable).as_option()?;

        let ast_id = ast.get("id")?.as_u64()? as usize;
        let name = ast.get("name")?.as_str()?.to_owned();
        let solc_location = solx_utils::DebugInfoSolcLocation::parse(
            ast.get("src")?.as_str()?,
            solx_utils::DebugInfoSolcLocationOrdering::Ast,
        )
        .ok()?;
        let source_code = sources
            .get(path)
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            path.to_owned(),
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
        path: &str,
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
            .get(path)
            .and_then(|source| source.content.as_deref());
        let mapped_location = solx_utils::DebugInfoMappedLocation::from_solc_location(
            path.to_owned(),
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
