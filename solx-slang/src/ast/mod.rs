//!
//! Slang AST construction from parsed compilation units.
//!

/// Source unit (top-level file) lowering to MLIR.
pub(crate) mod source_unit;

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
}
