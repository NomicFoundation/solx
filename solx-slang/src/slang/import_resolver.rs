//!
//! Import resolution for Slang compilation units.
//!

use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::compilation::ImportResolver;
use slang_solidity_v2::diagnostics::kinds::compilation::UnresolvedImport;

use solx_utils::Remapping;

/// Resolves import paths to source identifiers as solc does; Slang reports a resolved identifier
/// that is not among the compilation's sources as `MissingImportedFile`.
pub struct SourceImportResolver<'a> {
    /// Import remappings applied to resolved import paths, in input order.
    pub remappings: &'a [Remapping],
}

impl SourceImportResolver<'_> {
    /// solc's `util::absolutePath` (`libsolutil/CommonIO.cpp`): the import's components are
    /// appended to the importing file's identifier with its filename removed, `..` taking the
    /// boost `parent_path` at each step. Identifiers are `/`-separated strings handled as the
    /// POSIX flavour of `boost::filesystem::path` (v3, boost 1.83) does, so `\\` is an ordinary
    /// character and the `//` in URL identifiers survives.
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

    /// The size of a `//name` root name, from `find_root_directory_start`
    /// (`libs/filesystem/src/path.cpp`), which is also where its root directory starts. Any other
    /// path has no root name, and its root directory, when it has one, at 0.
    fn root_name_size(path: &str) -> usize {
        match path.as_bytes() {
            [b'/', b'/', b'/', ..] => 0,
            [b'/', b'/', name @ ..] => 2 + name.iter().take_while(|byte| **byte != b'/').count(),
            _ => 0,
        }
    }

    /// `find_parent_path_size` from the same file: drops the filename and the separators before
    /// it, keeping a root directory only when a filename followed it, and a root name whole.
    fn parent_path(path: &str) -> &str {
        let bytes = path.as_bytes();
        let root_size = Self::root_name_size(path);
        let filename_size = bytes[root_size..]
            .iter()
            .rev()
            .take_while(|byte| **byte != b'/')
            .count();
        let mut end_pos = bytes.len() - filename_size;
        loop {
            if end_pos == root_size {
                end_pos = 0;
                break;
            }
            end_pos -= 1;
            if bytes[end_pos] != b'/' {
                end_pos += 1;
                break;
            }
            if end_pos == root_size {
                end_pos += usize::from(filename_size > 0);
                break;
            }
        }
        &path[..end_pos]
    }

    /// Whether boost's `filename()` is the root directory (`filename_v3`, `is_root_separator`):
    /// the path ends with a separator that, past duplicates, is the root directory's.
    fn ends_with_root_separator(path: &str) -> bool {
        let bytes = path.as_bytes();
        if bytes.last() != Some(&b'/') {
            return false;
        }
        let root_size = Self::root_name_size(path);
        let mut pos = bytes.len() - 1;
        while pos > root_size && bytes[pos - 1] == b'/' {
            pos -= 1;
        }
        pos == root_size
    }

    /// solc's `ImportRemapper::apply` (`libsolidity/interface/ImportRemapper.cpp`).
    fn apply_remappings(&self, source_file_id: &str, path: &str) -> String {
        let mut longest_context = 0;
        let mut longest_prefix = 0;
        let mut best_target = "";
        for remapping in self.remappings.iter() {
            if remapping.context.len() < longest_context {
                continue;
            }
            if !source_file_id.starts_with(remapping.context.as_str()) {
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
            best_target = remapping.target.as_str();
        }
        format!("{best_target}{}", &path[longest_prefix..])
    }
}

impl ImportResolver for SourceImportResolver<'_> {
    fn resolve_import(
        &mut self,
        source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        // `CompilerStack::resolveImports`: `applyRemapping(util::absolutePath(path, context))`.
        let resolved = if matches!(import_path.split('/').next(), Some("." | "..")) {
            Self::resolve_relative(source_file_id.as_str(), import_path)
        } else {
            import_path.to_owned()
        };
        Ok(FileId::from(self.apply_remappings(
            source_file_id.as_str(),
            resolved.as_str(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use slang_solidity_v2::compilation::FileId;
    use slang_solidity_v2::compilation::ImportResolver;

    use solx_utils::Remapping;

    use super::SourceImportResolver;

    fn remappings(remappings: &[&str]) -> Vec<Remapping> {
        remappings
            .iter()
            .map(|remapping| remapping.parse().expect("valid remapping"))
            .collect()
    }

    fn resolve(remappings: &[Remapping], from: &str, import: &str) -> String {
        SourceImportResolver { remappings }
            .resolve_import(&FileId::from(from), import)
            .expect("resolution is infallible")
            .to_string()
    }

    #[test]
    fn direct_import_remapped() {
        let remappings = remappings(&["@oz/=npm/oz@1.0.0/"]);
        assert_eq!(
            resolve(&remappings, "project/Main.sol", "@oz/A.sol"),
            "npm/oz@1.0.0/A.sol"
        );
    }

    #[test]
    fn context_scopes_remapping() {
        let remappings = remappings(&["project/:@oz/=npm/oz@1.0.0/"]);
        assert_eq!(
            resolve(&remappings, "project/Main.sol", "@oz/A.sol"),
            "npm/oz@1.0.0/A.sol"
        );
        assert_eq!(
            resolve(&remappings, "npm/dep@1.0.0/Main.sol", "@oz/A.sol"),
            "@oz/A.sol"
        );
    }

    #[test]
    fn unmatched_prefix_leaves_the_path() {
        let remappings = remappings(&["@oz/=npm/oz/"]);
        assert_eq!(resolve(&remappings, "Main.sol", "Other.sol"), "Other.sol");
    }

    #[test]
    fn longest_context_beats_longer_prefix() {
        for order in [
            ["lib/sub/=generic/lib/sub/", "project/:lib/=scoped/"],
            ["project/:lib/=scoped/", "lib/sub/=generic/lib/sub/"],
        ] {
            assert_eq!(
                resolve(&remappings(&order), "project/Main.sol", "lib/sub/A.sol"),
                "scoped/sub/A.sol"
            );
        }
    }

    #[test]
    fn longest_prefix_wins_within_context() {
        for order in [
            ["lib/=generic/", "lib/sub/=specific/"],
            ["lib/sub/=specific/", "lib/=generic/"],
        ] {
            assert_eq!(
                resolve(&remappings(&order), "Main.sol", "lib/sub/A.sol"),
                "specific/A.sol"
            );
        }
    }

    /// solc 0.8.34 with `["lib/=second/", "lib/=first/"]` compiles only when `first/A.sol` exists.
    #[test]
    fn later_remapping_wins_ties() {
        assert_eq!(
            resolve(
                &remappings(&["lib/=first/", "lib/=second/"]),
                "Main.sol",
                "lib/A.sol"
            ),
            "second/A.sol"
        );
        assert_eq!(
            resolve(
                &remappings(&["lib/=second/", "lib/=first/"]),
                "Main.sol",
                "lib/A.sol"
            ),
            "first/A.sol"
        );
    }

    /// solc 0.8.34 resolves `./A.sol` from `contracts/B.sol` to `contracts/A.sol` and only then
    /// applies `contracts/=lib/`.
    #[test]
    fn relative_import_remapped_after_resolution() {
        let remappings = remappings(&["contracts/=lib/"]);
        assert_eq!(
            resolve(&remappings, "contracts/B.sol", "./A.sol"),
            "lib/A.sol"
        );
    }

    #[test]
    fn relative_import_without_remappings() {
        assert_eq!(resolve(&[], "a/b.sol", "./m.sol"), "a/m.sol");
        assert_eq!(resolve(&[], "a/b.sol", "../x.sol"), "x.sol");
        assert_eq!(resolve(&[], "x.sol", "../x.sol"), "x.sol");
    }

    /// Sourcify stores Remix-style sources under their GitHub URLs.
    #[test]
    fn url_source_identifiers_keep_their_double_slash() {
        assert_eq!(
            resolve(
                &[],
                "https://github.com/oz/contracts/access/Ownable.sol",
                "../utils/Context.sol"
            ),
            "https://github.com/oz/contracts/utils/Context.sol"
        );
    }

    /// Foundry projects verified from a subdirectory key their libraries this way.
    #[test]
    fn source_identifiers_keep_leading_parent_dirs() {
        assert_eq!(
            resolve(
                &[],
                "../../lib/oz/contracts/access/Ownable.sol",
                "../utils/Context.sol"
            ),
            "../../lib/oz/contracts/utils/Context.sol"
        );
    }

    /// solc 0.8.34: `Source "x.sol" not found`, so a `..` out of the root directory yields a bare
    /// identifier, not `/x.sol`.
    #[test]
    fn climbing_out_of_the_root_directory_drops_it() {
        assert_eq!(resolve(&[], "/a.sol", "../x.sol"), "x.sol");
        assert_eq!(resolve(&[], "/a/b.sol", "../../x.sol"), "x.sol");
    }

    #[test]
    fn root_directory_is_kept_below_it() {
        assert_eq!(resolve(&[], "/Main.sol", "./Dep.sol"), "/Dep.sol");
        assert_eq!(resolve(&[], "/a", "./b"), "/b");
    }

    /// solc 0.8.34 resolves `./a.sol` from `/` to `/a.sol` and from `///` to `///a.sol`:
    /// `absolutePath` skips `remove_filename` when the filename is the root directory.
    #[test]
    fn source_identifier_that_is_the_root_directory_keeps_it() {
        assert_eq!(resolve(&[], "/", "./a.sol"), "/a.sol");
        assert_eq!(resolve(&[], "///", "./a.sol"), "///a.sol");
    }

    /// solc 0.8.34: `Source "a/x.sol" not found` from `a/b/`, so a trailing separator that is not
    /// the root directory's goes the way of a filename.
    #[test]
    fn trailing_separator_source_identifier_is_removed_like_a_filename() {
        assert_eq!(resolve(&[], "a/b/", "../x.sol"), "a/x.sol");
    }

    /// solc 0.8.34: `Source "x.sol" not found` from `///` and `/x.sol` from `///a/b.sol`, where a
    /// `//` root name would have kept `//` and `///` respectively.
    #[test]
    fn three_leading_separators_are_a_root_directory_not_a_root_name() {
        assert_eq!(resolve(&[], "///", "../x.sol"), "x.sol");
        assert_eq!(resolve(&[], "///a/b.sol", "../x.sol"), "/x.sol");
    }

    /// solc 0.8.34: `Source "a.sol" not found`, so a root name with nothing after it is not kept
    /// as the parent.
    #[test]
    fn bare_root_name_has_no_parent() {
        assert_eq!(resolve(&[], "//server", "./a.sol"), "a.sol");
    }

    /// boost treats a leading `//name` as a root name that `..` cannot climb out of.
    #[test]
    fn network_root_name_is_kept_whole() {
        assert_eq!(resolve(&[], "//a/b.sol", "../x.sol"), "//a/x.sol");
    }

    /// Each `..` drops one component together with the run of separators before it, so the
    /// `//` after the URL scheme is stepped over in one move.
    #[test]
    fn climbing_through_a_double_slash_counts_it_once() {
        assert_eq!(
            resolve(&[], "https://github.com/o/c/A.sol", "../../../../x.sol"),
            "x.sol"
        );
    }

    /// A trailing separator is boost's implicit `.` element and adds nothing.
    #[test]
    fn trailing_separator_import_resolves_to_the_directory() {
        assert_eq!(resolve(&[], "a/b/c.sol", "../"), "a");
    }

    #[test]
    fn non_relative_import_does_not_resolve_against_importing_directory() {
        assert_eq!(resolve(&[], "dir/B.sol", "A.sol"), "A.sol");
    }

    /// `util::absolutePath` compares the first component with `.` and `..`, not its first byte.
    #[test]
    fn dot_prefixed_first_component_is_not_relative() {
        assert_eq!(resolve(&[], "dir/B.sol", ".hidden/A.sol"), ".hidden/A.sol");
    }

    /// CLI input paths become source identifiers verbatim, so `solx ./b.sol ./a.sol` registers
    /// `./b.sol` and `./a.sol`.
    #[test]
    fn dot_prefixed_source_identifiers_resolve_directly() {
        assert_eq!(resolve(&[], "./b.sol", "./a.sol"), "./a.sol");
    }
}
