//!
//! `solx` Sourcify sweep corpus release.
//!

///
/// The pinned corpus release.
///
#[derive(Debug, serde::Deserialize)]
pub struct Corpus {
    /// The `.tar.xz` release asset URL.
    pub url: String,
    /// The SHA-256 of the release asset.
    pub sha256: String,
}
