//!
//! The `solc --standard-json` output source.
//!

use std::collections::BTreeMap;

use boolinator::Boolinator;

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
    /// Returns the list of messages for some specific parts of the AST.
    ///
    pub fn get_ast_nodes<K, V, F>(getter: &F, path: &str, ast: &serde_json::Value) -> BTreeMap<K, V>
    where
        K: std::cmp::Ord,
        V: solx_utils::IDebugInfoAstNode<Key = K>,
        F: Fn(&str, &serde_json::Value) -> Option<V>,
    {
        let mut ast_nodes = BTreeMap::new();
        if let Some(ast_node) = getter(path, ast) {
            ast_nodes.insert(ast_node.index_id(), ast_node);
        }

        match ast {
            serde_json::Value::Array(array) => {
                for element in array.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, path, element));
                }
            }
            serde_json::Value::Object(object) => {
                for (_key, value) in object.iter() {
                    ast_nodes.extend(Self::get_ast_nodes(getter, path, value));
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
        line_index: &solx_utils::DebugInfoLineIndex<'_>,
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
        let mapped_location =
            line_index.mapped_location(path.to_owned(), solc_location.start, solc_location.end);

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
        line_index: &solx_utils::DebugInfoLineIndex<'_>,
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
        let mapped_location =
            line_index.mapped_location(path.to_owned(), solc_location.start, solc_location.end);

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
        line_index: &solx_utils::DebugInfoLineIndex<'_>,
    ) -> Option<solx_utils::DebugInfoAstNode> {
        let ast = ast.as_object()?;

        // Yul nodes (inside `InlineAssembly.AST`) carry no `id`, but their
        // `src` points into the Solidity source like any other node's.
        let ast_id = match ast.get("id") {
            Some(id) => Some(id.as_u64()? as usize),
            None => {
                (ast.get("nodeType")?.as_str()?.starts_with("Yul")).as_option()?;
                None
            }
        };
        let solc_location = solx_utils::DebugInfoSolcLocation::parse(
            ast.get("src")?.as_str()?,
            solx_utils::DebugInfoSolcLocationOrdering::Ast,
        )
        .ok()?;
        let mapped_location =
            line_index.mapped_location(path.to_owned(), solc_location.start, solc_location.end);

        Some(solx_utils::DebugInfoAstNode::new(
            ast_id,
            solc_location,
            mapped_location,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Source;

    fn ast_node(ast: serde_json::Value) -> Option<solx_utils::DebugInfoAstNode> {
        let source_code = "contract C {}\n";
        let line_index = solx_utils::DebugInfoLineIndex::new(source_code);
        Source::ast_node("Test.sol", &ast, &line_index)
    }

    /// Yul nodes inside `InlineAssembly.AST` carry no `id`; rejecting them drops the
    /// per-opcode source refs that solc emits for `assembly {}` bodies.
    #[test]
    fn resolves_yul_node_without_id() {
        let node = ast_node(serde_json::json!({
            "nodeType": "YulFunctionCall",
            "src": "0:8:0",
        }))
        .expect("Yul nodes without an `id` must resolve");
        assert_eq!(node.ast_id, None);
    }

    #[test]
    fn resolves_solidity_node_with_id() {
        let node = ast_node(serde_json::json!({
            "nodeType": "ExpressionStatement",
            "id": 7,
            "src": "0:8:0",
        }))
        .expect("Solidity nodes with an `id` must resolve");
        assert_eq!(node.ast_id, Some(7));
    }

    #[test]
    fn rejects_solidity_node_without_id() {
        assert!(
            ast_node(serde_json::json!({
                "nodeType": "ExpressionStatement",
                "src": "0:8:0",
            }))
            .is_none()
        );
    }
}
