//!
//! Slang AST construction from parsed compilation units.
//!

use std::collections::BTreeMap;
use std::rc::Rc;

use slang_solidity::backend::ir::ast::SourceUnit;
use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::compilation::CompilationUnit;

/// Semantic ASTs produced from a Slang compilation unit.
///
/// Uses the `SemanticAnalysis` layer to obtain `ir::ast` nodes with
/// `Rc`-wrapped types, accessor methods, and name resolution support.
pub struct SemanticAst {
    semantic: Rc<SemanticAnalysis>,
    file_ids: Vec<String>,
}

impl SemanticAst {
    /// Builds the semantic AST from a compilation unit.
    ///
    /// Runs the full Slang semantic analysis pipeline (AST construction,
    /// contract flattening, definition collection, linearisation,
    /// type resolution, reference resolution) and caches the result.
    pub fn build(unit: &CompilationUnit) -> Self {
        let semantic = Rc::clone(unit.semantic_analysis());
        let file_ids: Vec<String> = unit.files().iter().map(|f| f.id().to_owned()).collect();
        Self { semantic, file_ids }
    }

    /// Returns the semantic AST root for a given file identifier.
    pub fn file_ast(&self, file_id: &str) -> Option<SourceUnit> {
        self.semantic.get_file_ast_root(file_id)
    }

    /// Returns the file identifiers in this AST.
    pub fn file_ids(&self) -> &[String] {
        &self.file_ids
    }

    /// Produces stub AST JSON entries for each file in this AST.
    ///
    /// The `ir::ast` types do not implement `Serialize` yet, so this
    /// is a placeholder until proper AST JSON serialization is available.
    ///
    /// TODO: fix when Slang AST implements `Serialize`.
    pub fn stub_ast_jsons(&self) -> BTreeMap<String, Option<serde_json::Value>> {
        self.file_ids
            .iter()
            .map(|path| {
                let stub = serde_json::json!({
                    "nodeType": "SourceUnit",
                    "src": path,
                    "_note": "AST JSON is a stub. Slang frontend is under construction."
                });
                (path.clone(), Some(stub))
            })
            .collect()
    }
}
