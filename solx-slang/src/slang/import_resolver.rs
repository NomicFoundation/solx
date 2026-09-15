//!
//! Import resolution for the Slang frontend.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::compilation::ImportResolver;
use slang_solidity_v2::diagnostics::kinds::compilation::UnresolvedImport;

use solx_utils::Remapping;

/// Resolves import paths to the source files provided to the compilation.
pub struct SourceImportResolver<'a> {
    /// The source files keyed by identifier; an import resolves only to one of them.
    pub sources: &'a BTreeMap<FileId, &'a str>,
    /// Import remappings applied to resolved import paths, as in solc.
    pub remappings: &'a [Remapping],
}

impl SourceImportResolver<'_> {
    ///
    /// solc's `util::absolutePath` (`libsolutil/CommonIO.cpp`): the import's components are
    /// appended to the importing file's identifier with its filename removed, `..` taking the
    /// boost `parent_path` at each step. Identifiers are `/`-separated strings handled as the
    /// POSIX flavour of `boost::filesystem::path` (v3, boost 1.83) does, so `\\` is an ordinary
    /// character and the `//` in URL identifiers survives.
    ///
    fn resolve_relative(source_file_id: &str, import_path: &str) -> String {
        // `absolutePath` skips `remove_filename` when the filename is the root directory itself.
        let mut resolved = if Self::ends_with_root_separator(source_file_id) {
            source_file_id.to_owned()
        } else {
            Self::parent_path(source_file_id).to_owned()
        };
        // boost's path iterator yields `.` for a trailing separator and skips repeated ones.
        for component in import_path.split('/') {
            match component {
                ".." => {
                    let parent_length = Self::parent_path(resolved.as_str()).len();
                    resolved.truncate(parent_length);
                }
                "." | "" => {}
                component => {
                    if !resolved.is_empty() && !resolved.ends_with('/') {
                        resolved.push('/');
                    }
                    resolved.push_str(component);
                }
            }
        }
        resolved
    }

    ///
    /// The root of a POSIX boost path as `(root_name_size, root_dir_pos)`, from
    /// `find_root_directory_start` (`libs/filesystem/src/path.cpp`): `//name` is a root name up to
    /// the next separator, one or three-plus leading `/` are a root directory at 0, and
    /// `root_dir_pos == len` means there is none.
    ///
    fn root(path: &str) -> (usize, usize) {
        let bytes = path.as_bytes();
        match bytes {
            [] => (0, 0),
            [b'/', b'/'] => (2, 2),
            [b'/', b'/', b'/', ..] => (0, 0),
            [b'/', b'/', ..] => {
                let root_name_size = bytes[2..]
                    .iter()
                    .position(|byte| *byte == b'/')
                    .map_or(bytes.len(), |index| index + 2);
                (root_name_size, root_name_size)
            }
            [b'/', ..] => (0, 0),
            _ => (0, bytes.len()),
        }
    }

    ///
    /// `find_parent_path_size` from the same file: drops the filename and the separators before
    /// it, keeping a root directory only when a filename followed it, and a root name whole.
    ///
    fn parent_path(path: &str) -> &str {
        let bytes = path.as_bytes();
        let (root_name_size, root_dir_pos) = Self::root(path);
        let filename_size = bytes[root_name_size..]
            .iter()
            .rev()
            .take_while(|byte| **byte != b'/')
            .count();
        let mut end_pos = bytes.len() - filename_size;
        loop {
            if end_pos <= root_name_size {
                if filename_size == 0 {
                    end_pos = 0;
                }
                break;
            }
            end_pos -= 1;
            if bytes[end_pos] != b'/' {
                end_pos += 1;
                break;
            }
            if end_pos == root_dir_pos {
                end_pos += usize::from(filename_size > 0);
                break;
            }
        }
        &path[..end_pos]
    }

    ///
    /// Whether boost's `filename()` is the root directory (`filename_v3`, `is_root_separator`):
    /// the path ends with a separator that, past duplicates, is the root directory's.
    ///
    fn ends_with_root_separator(path: &str) -> bool {
        let bytes = path.as_bytes();
        let (_, root_dir_pos) = Self::root(path);
        if root_dir_pos >= bytes.len() || bytes.last() != Some(&b'/') {
            return false;
        }
        let mut pos = bytes.len() - 1;
        while pos > root_dir_pos && bytes[pos - 1] == b'/' {
            pos -= 1;
        }
        pos == root_dir_pos
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

impl ImportResolver for SourceImportResolver<'_> {
    fn resolve_import(
        &mut self,
        source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        // solc semantics (`CompilerStack::resolveImports`): an import with a leading
        // `.` or `..` component resolves against the importing file first; any other
        // path is taken verbatim; remappings then rewrite the result.
        let resolved = if matches!(import_path.split('/').next(), Some("." | "..")) {
            Self::resolve_relative(source_file_id.as_str(), import_path)
        } else {
            import_path.to_owned()
        };
        let remapped = self.apply_remappings(source_file_id.as_str(), resolved.as_str());
        let key = FileId::from(remapped.as_str());
        if self.sources.contains_key(&key) {
            return Ok(key);
        }

        Err(UnresolvedImport {
            reason: format!("failed to resolve import {import_path} in {source_file_id}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use slang_solidity_v2::compilation::FileId;
    use slang_solidity_v2::compilation::ImportResolver;

    use solx_utils::Remapping;

    use super::SourceImportResolver;

    /// Builds empty sources under the given identifiers, with the given remappings parsed.
    fn config(
        source_ids: &[&'static str],
        remappings: &[&str],
    ) -> (BTreeMap<FileId, &'static str>, Vec<Remapping>) {
        let sources = source_ids
            .iter()
            .map(|source_id| (FileId::from(*source_id), ""))
            .collect();
        let remappings = remappings
            .iter()
            .map(|remapping| remapping.parse().expect("valid remapping"))
            .collect();
        (sources, remappings)
    }

    /// Resolves an import and returns the resulting identifier, if any.
    fn resolve(
        sources: &BTreeMap<FileId, &str>,
        remappings: &[Remapping],
        from: &str,
        import: &str,
    ) -> Option<String> {
        SourceImportResolver {
            sources,
            remappings,
        }
        .resolve_import(&FileId::from(from), import)
        .ok()
        .map(|file_id| file_id.to_string())
    }

    #[test]
    fn direct_import_remapped() {
        let (sources, remappings) = config(
            &["project/Main.sol", "npm/oz@1.0.0/A.sol"],
            &["@oz/=npm/oz@1.0.0/"],
        );
        assert_eq!(
            resolve(&sources, &remappings, "project/Main.sol", "@oz/A.sol"),
            Some("npm/oz@1.0.0/A.sol".to_owned())
        );
    }

    #[test]
    fn context_scopes_remapping() {
        let (sources, remappings) = config(
            &[
                "project/Main.sol",
                "npm/dep@1.0.0/Main.sol",
                "npm/oz@1.0.0/A.sol",
            ],
            &["project/:@oz/=npm/oz@1.0.0/"],
        );
        assert_eq!(
            resolve(&sources, &remappings, "project/Main.sol", "@oz/A.sol"),
            Some("npm/oz@1.0.0/A.sol".to_owned())
        );
        assert_eq!(
            resolve(&sources, &remappings, "npm/dep@1.0.0/Main.sol", "@oz/A.sol"),
            None
        );
    }

    #[test]
    fn longest_context_beats_longer_prefix() {
        let (sources, remappings) = config(
            &[
                "project/Main.sol",
                "generic/lib/sub/A.sol",
                "scoped/sub/A.sol",
            ],
            &["lib/sub/=generic/lib/sub/", "project/:lib/=scoped/"],
        );
        assert_eq!(
            resolve(&sources, &remappings, "project/Main.sol", "lib/sub/A.sol"),
            Some("scoped/sub/A.sol".to_owned())
        );
    }

    #[test]
    fn longest_prefix_wins_within_context() {
        let (sources, remappings) = config(
            &["Main.sol", "specific/A.sol", "generic/sub/A.sol"],
            &["lib/=generic/", "lib/sub/=specific/"],
        );
        assert_eq!(
            resolve(&sources, &remappings, "Main.sol", "lib/sub/A.sol"),
            Some("specific/A.sol".to_owned())
        );
    }

    #[test]
    fn later_remapping_wins_ties() {
        let (sources, remappings) = config(
            &["Main.sol", "second/A.sol"],
            &["lib/=first/", "lib/=second/"],
        );
        assert_eq!(
            resolve(&sources, &remappings, "Main.sol", "lib/A.sol"),
            Some("second/A.sol".to_owned())
        );
    }

    #[test]
    fn relative_import_remapped_after_resolution() {
        // `./A.sol` from `contracts/B.sol` resolves to `contracts/A.sol`, which the
        // remapping then rewrites.
        let (sources, remappings) = config(&["contracts/B.sol", "lib/A.sol"], &["contracts/=lib/"]);
        assert_eq!(
            resolve(&sources, &remappings, "contracts/B.sol", "./A.sol"),
            Some("lib/A.sol".to_owned())
        );
    }

    #[test]
    fn relative_import_without_remappings() {
        let (sources, remappings) = config(&["a/b.sol", "a/m.sol", "x.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "a/b.sol", "./m.sol"),
            Some("a/m.sol".to_owned())
        );
        assert_eq!(
            resolve(&sources, &remappings, "a/b.sol", "../x.sol"),
            Some("x.sol".to_owned())
        );
        assert_eq!(
            resolve(&sources, &remappings, "x.sol", "../x.sol"),
            Some("x.sol".to_owned())
        );
    }

    #[test]
    fn bare_filename_direct_import() {
        let (sources, remappings) = config(&["Factory.sol", "Storage.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "Factory.sol", "Storage.sol"),
            Some("Storage.sol".to_owned())
        );
    }

    #[test]
    fn url_source_identifiers_keep_their_double_slash() {
        // Sourcify stores Remix-style sources under their GitHub URLs.
        let (sources, remappings) = config(
            &[
                "https://github.com/oz/contracts/access/Ownable.sol",
                "https://github.com/oz/contracts/utils/Context.sol",
            ],
            &[],
        );
        assert_eq!(
            resolve(
                &sources,
                &remappings,
                "https://github.com/oz/contracts/access/Ownable.sol",
                "../utils/Context.sol"
            ),
            Some("https://github.com/oz/contracts/utils/Context.sol".to_owned())
        );
    }

    #[test]
    fn source_identifiers_keep_leading_parent_dirs() {
        // Foundry projects verified from a subdirectory key their libraries this way.
        let (sources, remappings) = config(
            &[
                "../../lib/oz/contracts/access/Ownable.sol",
                "../../lib/oz/contracts/utils/Context.sol",
            ],
            &[],
        );
        assert_eq!(
            resolve(
                &sources,
                &remappings,
                "../../lib/oz/contracts/access/Ownable.sol",
                "../utils/Context.sol"
            ),
            Some("../../lib/oz/contracts/utils/Context.sol".to_owned())
        );
    }

    /// solc 0.8.34: `Source "x.sol" not found`, so a `..` out of the root directory yields a bare
    /// identifier, not `/x.sol`.
    #[test]
    fn climbing_out_of_the_root_directory_drops_it() {
        let (sources, remappings) = config(&["/a.sol", "/a/b.sol", "x.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "/a.sol", "../x.sol"),
            Some("x.sol".to_owned())
        );
        assert_eq!(
            resolve(&sources, &remappings, "/a/b.sol", "../../x.sol"),
            Some("x.sol".to_owned())
        );
    }

    #[test]
    fn root_directory_is_kept_below_it() {
        let (sources, remappings) = config(&["/Main.sol", "/Dep.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "/Main.sol", "./Dep.sol"),
            Some("/Dep.sol".to_owned())
        );
    }

    /// boost treats a leading `//name` as a root name that `..` cannot climb out of.
    #[test]
    fn network_root_name_is_kept_whole() {
        let (sources, remappings) = config(&["//a/b.sol", "//a/x.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "//a/b.sol", "../x.sol"),
            Some("//a/x.sol".to_owned())
        );
    }

    /// Each `..` drops one component together with the run of separators before it, so the
    /// `//` after the URL scheme is stepped over in one move.
    #[test]
    fn climbing_through_a_double_slash_counts_it_once() {
        let (sources, remappings) = config(&["https://github.com/o/c/A.sol", "x.sol"], &[]);
        assert_eq!(
            resolve(
                &sources,
                &remappings,
                "https://github.com/o/c/A.sol",
                "../../../../x.sol"
            ),
            Some("x.sol".to_owned())
        );
    }

    /// A trailing separator is boost's implicit `.` element and adds nothing.
    #[test]
    fn trailing_separator_import_resolves_to_the_directory() {
        let (sources, remappings) = config(&["a/b/c.sol", "a"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "a/b/c.sol", "../"),
            Some("a".to_owned())
        );
    }

    #[test]
    fn non_relative_import_does_not_resolve_against_importing_directory() {
        let (sources, remappings) = config(&["dir/B.sol", "dir/A.sol"], &[]);
        assert_eq!(resolve(&sources, &remappings, "dir/B.sol", "A.sol"), None);
    }

    #[test]
    fn dot_prefixed_source_identifiers_resolve_directly() {
        // CLI input paths become source identifiers verbatim, so `solx ./b.sol ./a.sol`
        // registers `./b.sol` and `./a.sol`.
        let (sources, remappings) = config(&["./b.sol", "./a.sol"], &[]);
        assert_eq!(
            resolve(&sources, &remappings, "./b.sol", "./a.sol"),
            Some("./a.sol".to_owned())
        );
    }

    #[test]
    fn remapped_import_does_not_fall_back_to_its_original_path() {
        let (sources, remappings) = config(&["Main.sol", "lib/A.sol"], &["lib/=x/"]);
        assert_eq!(
            resolve(&sources, &remappings, "Main.sol", "lib/A.sol"),
            None
        );
    }

    #[test]
    fn unresolved_import() {
        let (sources, remappings) = config(&["Main.sol"], &["@oz/=npm/oz@1.0.0/"]);
        assert_eq!(
            resolve(&sources, &remappings, "Main.sol", "@oz/A.sol"),
            None
        );
        assert_eq!(
            resolve(&sources, &remappings, "Main.sol", "missing.sol"),
            None
        );
    }
}
