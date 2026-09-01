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

/// An import remapping in solc's `[context:]prefix=target` form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remapping {
    /// Applies only within files whose identifier starts with this; empty matches every file.
    pub context: String,
    /// The import path prefix that `target` replaces.
    pub prefix: String,
    /// The replacement for `prefix`.
    pub target: String,
}

impl Remapping {
    ///
    /// Parses solc's `[context:]prefix=target` form.
    ///
    /// Returns `None` for an invalid remapping: no `=`, or an empty prefix,
    /// as in solc's `ImportRemapper::parseRemapping`.
    ///
    pub fn parse(remapping: &str) -> Option<Self> {
        let (rest, target) = remapping.split_once('=')?;
        let (context, prefix) = match rest.split_once(':') {
            Some((context, prefix)) => (context, prefix),
            None => ("", rest),
        };
        if prefix.is_empty() {
            return None;
        }
        Some(Self {
            context: context.to_owned(),
            prefix: prefix.to_owned(),
            target: target.to_owned(),
        })
    }
}

/// Provides file reading and import resolution for the Slang compilation builder.
pub struct CompilationConfig {
    /// The file contents keyed by identifier, for reading and import resolution.
    pub sources: BTreeMap<FileId, String>,
    /// Import remappings applied to resolved import paths, as in solc.
    pub remappings: Vec<Remapping>,
}

impl CompilationConfig {
    /// Creates a new configuration from a map of file identifiers to source contents.
    pub fn new(sources: BTreeMap<FileId, String>, remappings: Vec<Remapping>) -> Self {
        Self {
            sources,
            remappings,
        }
    }

    ///
    /// Resolves a relative import against the importing file's directory,
    /// normalizing `.` and `..` components.
    ///
    fn resolve_relative(source_file_id: &str, import_path: &str) -> String {
        let dir = Path::new(source_file_id)
            .parent()
            .unwrap_or_else(|| Path::new(""));
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
        clean.to_string_lossy().replace('\\', "/")
    }

    ///
    /// Applies the best matching remapping to `path`, mirroring solc's
    /// `ImportRemapper::apply`: the longest matching context wins, then the
    /// longest matching prefix; on ties the later remapping wins. Without a
    /// match the path is returned unchanged.
    ///
    fn apply_remappings(&self, context: &str, path: &str) -> String {
        let mut longest_context = 0;
        let mut longest_prefix = 0;
        let mut best_target = None;
        for remapping in self.remappings.iter() {
            if remapping.context.len() < longest_context {
                continue;
            }
            if !context.starts_with(remapping.context.as_str()) {
                continue;
            }
            if remapping.prefix.len() < longest_prefix && remapping.context.len() == longest_context
            {
                continue;
            }
            if !path.starts_with(remapping.prefix.as_str()) {
                continue;
            }
            longest_context = remapping.context.len();
            longest_prefix = remapping.prefix.len();
            best_target = Some(remapping.target.as_str());
        }
        match best_target {
            Some(target) => format!("{target}{}", &path[longest_prefix..]),
            None => path.to_owned(),
        }
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
        // solc semantics (`CompilerStack::resolveImports`): an import with a leading
        // `.` or `..` component resolves against the importing file first; any other
        // path is taken verbatim; remappings then rewrite the result.
        let resolved = if matches!(
            Path::new(import_path).components().next(),
            Some(Component::CurDir | Component::ParentDir)
        ) {
            Self::resolve_relative(source_file_id.as_str(), import_path)
        } else {
            import_path.to_owned()
        };
        let remapped = self.apply_remappings(source_file_id.as_str(), resolved.as_str());
        let key = FileId::from(remapped.as_str());
        if self.sources.contains_key(&key) {
            return Ok(key);
        }

        // Sources can be registered under non-normalized identifiers such as
        // `./main.sol`; fall back to the verbatim import string.
        let verbatim = FileId::from(import_path);
        if self.sources.contains_key(&verbatim) {
            return Ok(verbatim);
        }

        Err(UnresolvedImport {
            reason: format!("failed to resolve import {import_path} in {source_file_id}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use slang_solidity_v2::compilation::CompilationBuilderConfig;
    use slang_solidity_v2::compilation::FileId;

    use super::CompilationConfig;
    use super::Remapping;

    /// Builds a configuration with empty sources under the given identifiers.
    fn config(source_ids: &[&str], remappings: &[&str]) -> CompilationConfig {
        let sources: BTreeMap<FileId, String> = source_ids
            .iter()
            .map(|id| (FileId::from(*id), String::new()))
            .collect();
        let remappings = remappings
            .iter()
            .map(|remapping| Remapping::parse(remapping).expect("valid remapping"))
            .collect();
        CompilationConfig::new(sources, remappings)
    }

    /// Resolves an import and returns the resulting identifier, if any.
    fn resolve(config: &mut CompilationConfig, from: &str, import: &str) -> Option<String> {
        config
            .resolve_import(&FileId::from(from), import)
            .ok()
            .map(|file_id| file_id.to_string())
    }

    #[test]
    fn parse() {
        assert_eq!(
            Remapping::parse("@oz/=npm/oz@1.0.0/"),
            Some(Remapping {
                context: "".to_owned(),
                prefix: "@oz/".to_owned(),
                target: "npm/oz@1.0.0/".to_owned(),
            })
        );
        assert_eq!(
            Remapping::parse("project/:@dep/=npm/dep@1.2.3/"),
            Some(Remapping {
                context: "project/".to_owned(),
                prefix: "@dep/".to_owned(),
                target: "npm/dep@1.2.3/".to_owned(),
            })
        );
        // The prefix may contain colons past the first separator.
        assert_eq!(
            Remapping::parse("a:b:c=d"),
            Some(Remapping {
                context: "a".to_owned(),
                prefix: "b:c".to_owned(),
                target: "d".to_owned(),
            })
        );
        // An empty target erases the prefix.
        assert_eq!(
            Remapping::parse("lib/="),
            Some(Remapping {
                context: "".to_owned(),
                prefix: "lib/".to_owned(),
                target: "".to_owned(),
            })
        );
        assert_eq!(Remapping::parse("no-equals-sign"), None);
        assert_eq!(Remapping::parse("=target"), None);
        assert_eq!(Remapping::parse(":=target"), None);
    }

    #[test]
    fn direct_import_remapped() {
        let mut config = config(
            &["project/Main.sol", "npm/oz@1.0.0/A.sol"],
            &["@oz/=npm/oz@1.0.0/"],
        );
        assert_eq!(
            resolve(&mut config, "project/Main.sol", "@oz/A.sol"),
            Some("npm/oz@1.0.0/A.sol".to_owned())
        );
    }

    #[test]
    fn context_scopes_remapping() {
        let mut config = config(
            &[
                "project/Main.sol",
                "npm/dep@1.0.0/Main.sol",
                "npm/oz@1.0.0/A.sol",
            ],
            &["project/:@oz/=npm/oz@1.0.0/"],
        );
        assert_eq!(
            resolve(&mut config, "project/Main.sol", "@oz/A.sol"),
            Some("npm/oz@1.0.0/A.sol".to_owned())
        );
        assert_eq!(
            resolve(&mut config, "npm/dep@1.0.0/Main.sol", "@oz/A.sol"),
            None
        );
    }

    #[test]
    fn longest_context_beats_longer_prefix() {
        let mut config = config(
            &[
                "project/Main.sol",
                "generic/lib/sub/A.sol",
                "scoped/sub/A.sol",
            ],
            &["lib/sub/=generic/lib/sub/", "project/:lib/=scoped/"],
        );
        assert_eq!(
            resolve(&mut config, "project/Main.sol", "lib/sub/A.sol"),
            Some("scoped/sub/A.sol".to_owned())
        );
    }

    #[test]
    fn longest_prefix_wins_within_context() {
        let mut config = config(
            &["Main.sol", "specific/A.sol", "generic/sub/A.sol"],
            &["lib/=generic/", "lib/sub/=specific/"],
        );
        assert_eq!(
            resolve(&mut config, "Main.sol", "lib/sub/A.sol"),
            Some("specific/A.sol".to_owned())
        );
    }

    #[test]
    fn later_remapping_wins_ties() {
        let mut config = config(
            &["Main.sol", "second/A.sol"],
            &["lib/=first/", "lib/=second/"],
        );
        assert_eq!(
            resolve(&mut config, "Main.sol", "lib/A.sol"),
            Some("second/A.sol".to_owned())
        );
    }

    #[test]
    fn relative_import_remapped_after_resolution() {
        // Verified against solc: `./A.sol` from `contracts/B.sol` resolves to
        // `contracts/A.sol`, which the remapping then rewrites.
        let mut config = config(&["contracts/B.sol", "lib/A.sol"], &["contracts/=lib/"]);
        assert_eq!(
            resolve(&mut config, "contracts/B.sol", "./A.sol"),
            Some("lib/A.sol".to_owned())
        );
    }

    #[test]
    fn relative_import_without_remappings() {
        let mut config = config(&["a/b.sol", "a/m.sol", "x.sol"], &[]);
        assert_eq!(
            resolve(&mut config, "a/b.sol", "./m.sol"),
            Some("a/m.sol".to_owned())
        );
        assert_eq!(
            resolve(&mut config, "a/b.sol", "../x.sol"),
            Some("x.sol".to_owned())
        );
    }

    #[test]
    fn bare_filename_direct_import() {
        let mut config = config(&["Factory.sol", "Storage.sol"], &[]);
        assert_eq!(
            resolve(&mut config, "Factory.sol", "Storage.sol"),
            Some("Storage.sol".to_owned())
        );
    }

    #[test]
    fn verbatim_fallback_for_non_normalized_keys() {
        let mut config = config(&["./b.sol", "./a.sol"], &[]);
        assert_eq!(
            resolve(&mut config, "./b.sol", "./a.sol"),
            Some("./a.sol".to_owned())
        );
    }

    #[test]
    fn unresolved_import() {
        let mut config = config(&["Main.sol"], &["@oz/=npm/oz@1.0.0/"]);
        assert_eq!(resolve(&mut config, "Main.sol", "@oz/A.sol"), None);
        assert_eq!(resolve(&mut config, "Main.sol", "missing.sol"), None);
    }
}
