//!
//! Slang AST construction from parsed compilation units.
//!

use std::collections::BTreeMap;
use std::rc::Rc;

use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::backend::ir::ast::SourceUnit;
use slang_solidity::compilation::CompilationUnit;

/// Semantic ASTs produced from a Slang compilation unit.
///
/// Uses the `SemanticAnalysis` layer to obtain `ir::ast` nodes with
/// `Rc`-wrapped types, accessor methods, and name resolution support.
pub struct SemanticAst {
    semantic: Rc<SemanticAnalysis>,
    file_identifiers: Vec<String>,
}

impl SemanticAst {
    /// Wraps the semantic analysis already performed by the `CompilationUnit`.
    ///
    /// No analysis is run here — the `CompilationUnit` drives parsing and
    /// semantic analysis. This constructor captures a reference to the
    /// analysis result and collects the file identifiers.
    pub fn build(unit: &CompilationUnit) -> Self {
        let semantic = Rc::clone(unit.semantic_analysis());
        let file_identifiers: Vec<String> = unit
            .files()
            .iter()
            .map(|file| file.id().to_owned())
            .collect();
        Self {
            semantic,
            file_identifiers,
        }
    }

    /// Returns the semantic AST root for a given file identifier.
    ///
    /// # Returns None
    ///
    /// Returns `None` if the file identifier is not found in the semantic analysis.
    pub fn file_ast(&self, file_identifier: &str) -> Option<SourceUnit> {
        self.semantic.get_file_ast_root(file_identifier)
    }

    /// Returns the file identifiers in this AST.
    pub fn file_identifiers(&self) -> &[String] {
        &self.file_identifiers
    }

    /// Produces stub AST JSON entries for each file in this AST.
    ///
    /// The `ir::ast` types do not implement `Serialize` yet, so this
    /// is a placeholder until proper AST JSON serialization is available.
    ///
    /// TODO: fix when Slang AST implements `Serialize`.
    pub fn stub_ast_jsons(&self) -> BTreeMap<String, Option<serde_json::Value>> {
        self.file_identifiers
            .iter()
            .map(|path| {
                // Include contract names so solx-tester can find them.
                let mut contract_nodes = Vec::new();
                if let Some(source_unit) = self.file_ast(path) {
                    for member in source_unit.members().iter() {
                        if let slang_solidity::backend::ir::ast::SourceUnitMember::ContractDefinition(contract) = member {
                            contract_nodes.push(serde_json::json!({
                                "nodeType": "ContractDefinition",
                                "name": contract.name().name(),
                            }));
                        }
                    }
                }
                let stub = serde_json::json!({
                    "nodeType": "SourceUnit",
                    "src": path,
                    "nodes": contract_nodes,
                });
                (path.clone(), Some(stub))
            })
            .collect()
    }
}
