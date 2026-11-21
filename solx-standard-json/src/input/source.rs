//!
//! The `solc --standard-json` input source.
//!

use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use solx_utils::IPFSHash;
use solx_utils::Keccak256Hash;
use solx_utils::LineIndex;

///
/// The `solc --standard-json` input source.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code file content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The source file URLs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,

    /// The content's line index.
    #[serde(skip)]
    line_index: OnceLock<LineIndex>,
    /// The content's `keccak256` hash.
    #[serde(skip)]
    keccak256: OnceLock<Keccak256Hash>,
    /// The content's IPFS hash.
    #[serde(skip)]
    ipfs: OnceLock<IPFSHash>,
}

impl Source {
    /// Stdin input identifier.
    pub const STDIN_INPUT_IDENTIFIER: &'static str = "-";

    /// Stdin output identifier.
    pub const STDIN_OUTPUT_IDENTIFIER: &'static str = "<stdin>";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(content: String) -> Self {
        Self {
            content: Some(content),
            urls: None,
            line_index: OnceLock::new(),
            keccak256: OnceLock::new(),
            ipfs: OnceLock::new(),
        }
    }

    ///
    /// Reads the source from the file system.
    ///
    pub fn try_from_path(path: &Path) -> anyhow::Result<Self> {
        let content = if path.to_string_lossy() == Self::STDIN_INPUT_IDENTIFIER {
            std::io::read_to_string(std::io::stdin()).map_err(|error| {
                anyhow::anyhow!("{} reading: {error}", Self::STDIN_OUTPUT_IDENTIFIER)
            })
        } else {
            std::fs::read_to_string(path)
                .map_err(|error| anyhow::anyhow!("File {path:?} reading: {error}"))
        }?;

        Ok(Self::new(content))
    }

    ///
    /// Tries to resolve the source code.
    ///
    /// Only URLs pointing to the file system are supported.
    ///
    pub fn try_resolve(&mut self) -> anyhow::Result<()> {
        match (self.content.as_ref(), self.urls.as_ref()) {
            (Some(_), None) => Ok(()),
            (None, Some(urls)) => {
                if urls.is_empty() {
                    anyhow::bail!("At least one URL must be set.");
                }

                let mut errors = Vec::with_capacity(urls.len());
                for url in urls.iter() {
                    let url_path = PathBuf::from(url);
                    match Source::try_from_path(url_path.as_path()) {
                        Ok(resolved) => {
                            *self = resolved;
                            return Ok(());
                        }
                        Err(error) => errors.push(error),
                    }
                }
                anyhow::bail!(
                    "{}",
                    errors
                        .into_iter()
                        .map(|error| error.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                );
            }
            (Some(_), Some(_)) => anyhow::bail!("Both `content` and `urls` cannot be set."),
            (None, None) => anyhow::bail!("Either `content` or `urls` must be set."),
        }
    }

    ///
    /// Takes ownership of the source code and returns it.
    ///
    pub fn take_content(&mut self) -> Option<String> {
        self.content.take()
    }

    ///
    /// Returns the source code reference, if the source has been previously read or resolved.
    ///
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    ///
    /// The line index of the content, built on first use.
    ///
    pub fn line_index(&self) -> Option<&LineIndex> {
        let content = self.content.as_deref()?;
        Some(self.line_index.get_or_init(|| LineIndex::new(content)))
    }

    ///
    /// The `keccak256` hash of the content, computed on first use.
    ///
    pub fn keccak256(&self) -> Option<&Keccak256Hash> {
        let content = self.content.as_deref()?;
        Some(
            self.keccak256
                .get_or_init(|| Keccak256Hash::from_slice(content.as_bytes())),
        )
    }

    ///
    /// The IPFS hash of the content, computed on first use.
    ///
    pub fn ipfs(&self) -> Option<&IPFSHash> {
        let content = self.content.as_deref()?;
        Some(
            self.ipfs
                .get_or_init(|| IPFSHash::from_slice(content.as_bytes())),
        )
    }
}

impl From<&Path> for Source {
    fn from(path: &Path) -> Self {
        Self {
            content: None,
            urls: Some(vec![path.to_string_lossy().to_string()]),
            line_index: OnceLock::new(),
            keccak256: OnceLock::new(),
            ipfs: OnceLock::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Source;

    #[test]
    fn resolves_first_readable_url() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let missing = root.join("missing-source-does-not-exist.sol");
        let existing = root.join("Cargo.toml");
        let mut source = Source::from(existing.as_path());
        source.urls = Some(vec![
            missing.to_string_lossy().into_owned(),
            existing.to_string_lossy().into_owned(),
        ]);

        source.try_resolve().unwrap();

        let expected = std::fs::read_to_string(existing).unwrap();
        assert_eq!(source.content(), Some(expected.as_str()));
        assert!(source.urls.is_none());
    }

    #[test]
    fn reports_all_failed_urls() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let first = root.join("missing-source-one.sol");
        let second = root.join("missing-source-two.sol");
        let mut source = Source::from(first.as_path());
        source.urls = Some(vec![
            first.to_string_lossy().into_owned(),
            second.to_string_lossy().into_owned(),
        ]);

        let error = source.try_resolve().unwrap_err().to_string();

        assert!(error.contains(first.to_string_lossy().as_ref()));
        assert!(error.contains(second.to_string_lossy().as_ref()));
        assert!(source.content().is_none());
    }

    #[test]
    fn rejects_empty_urls() {
        let mut source = Source::from(Path::new("unused.sol"));
        source.urls = Some(Vec::new());

        let error = source.try_resolve().unwrap_err().to_string();

        assert_eq!(error, "At least one URL must be set.");
        assert!(source.content().is_none());
    }
}
