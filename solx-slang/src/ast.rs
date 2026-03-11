//!
//! Slang AST construction from parsed compilation units.
//!

use std::collections::BTreeMap;

use slang_solidity::backend::ir::ir2_flat_contracts::SourceUnit;
use slang_solidity::backend::passes::p0_build_ast;
use slang_solidity::backend::passes::p1_flatten_contracts;
use slang_solidity::compilation::CompilationUnit;

/// Flattened ASTs produced from a Slang compilation unit.
pub struct FlatAst {
    files: BTreeMap<String, SourceUnit>,
}

impl FlatAst {
    /// Builds the flattened AST for each file in the compilation unit.
    ///
    /// Runs the Slang `p0_build_ast` and `p1_flatten_contracts` passes to produce
    /// `ir2_flat_contracts::SourceUnit` per file.
    ///
    /// # Errors
    ///
    /// Returns an error if any file fails to produce a valid structured AST.
    pub fn build(unit: &CompilationUnit) -> anyhow::Result<Self> {
        let mut files = BTreeMap::new();

        for file in unit.files() {
            let structured = p0_build_ast::run_file(&file).ok_or_else(|| {
                anyhow::anyhow!("failed to build structured AST for '{}'", file.id())
            })?;
            let flattened = p1_flatten_contracts::run_file(unit.language_version(), &structured);
            files.insert(file.id().to_owned(), flattened);
        }

        Ok(Self { files })
    }

    /// Returns the flattened ASTs indexed by file identifier.
    pub fn files(&self) -> &BTreeMap<String, SourceUnit> {
        &self.files
    }

    /// Produces stub AST JSON entries for each file in this AST.
    ///
    /// The `ir2_flat_contracts` types do not implement `Serialize` yet, so this
    /// is a placeholder until proper AST JSON serialization is available.
    /// 
    /// TODO: fix when Slang AST implements `Serialize`.
    pub fn stub_ast_jsons(&self) -> BTreeMap<String, Option<serde_json::Value>> {
        self.files
            .keys()
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
