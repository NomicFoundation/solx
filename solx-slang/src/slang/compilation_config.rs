//!
//! Compilation builder configuration for the Slang frontend.
//!

use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use slang_solidity_v2::compilation::CompilationBuilderConfig;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::diagnostics::kinds::compilation::MissingFile;
use slang_solidity_v2::diagnostics::kinds::compilation::UnresolvedImport;

/// Provides file reading and import resolution for the Slang compilation builder.
pub struct CompilationConfig {
    /// The file contents keyed by identifier, for reading and import resolution.
    pub sources: BTreeMap<FileId, String>,
}

impl CompilationConfig {
    /// Creates a new configuration from a map of file identifiers to source contents.
    pub fn new(sources: BTreeMap<FileId, String>) -> Self {
        Self { sources }
    }
}

impl CompilationBuilderConfig for CompilationConfig {
    fn read_file(&mut self, file_id: &FileId) -> Result<String, MissingFile> {
        self.sources
            .get(file_id)
            .cloned()
            .ok_or_else(|| MissingFile {
                reason: format!("file not found {file_id}"),
            })
    }

    fn resolve_import(
        &mut self,
        source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        let candidate = FileId::from(import_path);
        if self.sources.contains_key(&candidate) {
            return Ok(candidate);
        }

        if let Some(dir) = Path::new(source_file_id.as_str()).parent() {
            let resolved = dir.join(import_path);
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
            let key = clean.to_string_lossy().replace('\\', "/").into();
            if self.sources.contains_key(&key) {
                return Ok(key);
            }
        }

        Err(UnresolvedImport {
            reason: format!("failed to resolve import {import_path} in {source_file_id}"),
        })
    }
}
