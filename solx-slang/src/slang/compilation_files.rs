//!
//! Import resolution for the Slang compilation: a path resolves to a file of the compilation by
//! its identifier, or relative to the importing file's.
//!

use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;

use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::compilation::ImportResolver;
use slang_solidity_v2::diagnostics::kinds::compilation::UnresolvedImport;

/// The sources of the compilation by file identifier, which an import path resolves against.
pub struct CompilationFiles<'sources>(pub &'sources BTreeMap<FileId, String>);

impl ImportResolver for CompilationFiles<'_> {
    fn resolve_import(
        &mut self,
        source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        let candidate = FileId::from(import_path);
        if self.0.contains_key(&candidate) {
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
            if self.0.contains_key(&key) {
                return Ok(key);
            }
        }

        Err(UnresolvedImport {
            reason: format!("failed to resolve import {import_path} in {source_file_id}"),
        })
    }
}
