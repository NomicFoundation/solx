//!
//! The `solc --standard-json` output contract EVM defined function.
//!

///
/// The `solc --standard-json` output contract EVM defined function.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinedFunction {
    /// The function name.
    pub name: String,
    /// The function AST node ID.
    #[serde(default)]
    pub ast_id: Option<usize>,
    /// The creation code function block tag.
    pub creation_tag: Option<usize>,
    /// The runtime code function block tag.
    pub runtime_tag: Option<usize>,
    /// The number of input arguments.
    #[serde(rename = "totalParamSize")]
    pub input_size: usize,
    /// The number of output arguments.
    #[serde(rename = "totalRetParamSize")]
    pub output_size: usize,
}
