//!
//! Compilation builder configuration for the Slang frontend.
//!

use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use slang_solidity::compilation::CompilationBuilderConfig;
use slang_solidity::cst::Cursor;

/// Provides file reading and import resolution for the Slang compilation builder.
pub struct SlangCompilationConfig {
    sources: BTreeMap<String, String>,
}

impl SlangCompilationConfig {
    /// Creates a new configuration from a map of file paths to source contents.
    pub fn new(sources: BTreeMap<String, String>) -> Self {
        Self { sources }
    }
}

impl CompilationBuilderConfig for SlangCompilationConfig {
    type Error = anyhow::Error;

    fn read_file(&mut self, file_identifier: &str) -> anyhow::Result<Option<String>> {
        Ok(self.sources.get(file_identifier).cloned())
    }

    fn resolve_import(
        &mut self,
        source_file_identifier: &str,
        import_path_cursor: &Cursor,
    ) -> anyhow::Result<Option<String>> {
        let literal = import_path_cursor.node().unparse();
        let path = literal.trim_matches(|character: char| character == '"' || character == '\'');

        // Try exact match first.
        if self.sources.contains_key(path) {
            return Ok(Some(path.to_owned()));
        }

        // Resolve relative imports against the importing file's directory.
        if let Some(dir) = Path::new(source_file_identifier).parent() {
            let resolved = dir.join(path);
            let mut normalized = Vec::new();
            for component in resolved.components() {
                match component {
                    Component::ParentDir => {
                        normalized.pop();
                    }
                    Component::CurDir => {}
                    other => normalized.push(other),
                }
            }
            let clean: std::path::PathBuf = normalized.into_iter().collect();
            let key = clean.to_string_lossy().to_string();
            if self.sources.contains_key(&key) {
                return Ok(Some(key));
            }
        }

        Ok(None)
    }
}
