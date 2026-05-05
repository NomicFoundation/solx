//!
//! Compilation builder configuration for the Slang frontend.
//!

use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use slang_solidity_v2::compilation::CompilationBuilderConfig;

/// Provides file reading and import resolution for the Slang compilation builder.
pub struct CompilationConfig {
    sources: BTreeMap<String, String>,
}

impl CompilationConfig {
    /// Creates a new configuration from a map of file paths to source contents.
    pub fn new(sources: BTreeMap<String, String>) -> Self {
        Self { sources }
    }
}

impl CompilationBuilderConfig for CompilationConfig {
    fn read_file(&mut self, file_identifier: &str) -> Result<String, String> {
        self.sources
            .get(file_identifier)
            .cloned()
            .ok_or(format!("file not found {file_identifier}"))
    }

    fn resolve_import(
        &mut self,
        source_file_identifier: &str,
        import_path: &str,
    ) -> Result<String, String> {
        let path = import_path
            .strip_prefix('"')
            .and_then(|stripped| stripped.strip_suffix('"'))
            .or_else(|| {
                import_path
                    .strip_prefix('\'')
                    .and_then(|stripped| stripped.strip_suffix('\''))
            })
            .unwrap_or(import_path);

        // Try exact match first.
        if self.sources.contains_key(path) {
            return Ok(path.to_owned());
        }

        // Resolve relative imports against the importing file's directory.
        if let Some(dir) = Path::new(source_file_identifier).parent() {
            let resolved = dir.join(path);
            let mut normalized = Vec::new();
            for component in resolved.components() {
                match component {
                    Component::ParentDir => {
                        if normalized.pop().is_none() {
                            normalized.push(component);
                        }
                    }
                    Component::CurDir => {}
                    other => normalized.push(other),
                }
            }
            let clean: std::path::PathBuf = normalized.into_iter().collect();
            let key = clean.to_string_lossy().replace('\\', "/");
            if self.sources.contains_key(&key) {
                return Ok(key);
            }
        }

        Err(format!(
            "failed to resolve import {import_path} in {source_file_identifier}"
        ))
    }
}
