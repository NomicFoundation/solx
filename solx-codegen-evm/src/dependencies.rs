//!
//! Collection of dependencies.
//!

///
/// This structure represents an ordered dependency collection
/// in the order they are encountered in IR from the top to the bottom.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    /// Top-level object identifier.
    pub identifier: String,
    /// List of EVM dependencies.
    pub inner: Vec<String>,
}

impl Dependencies {
    ///
    /// Create a new instance of dependencies.
    ///
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            inner: Vec::new(),
        }
    }

    ///
    /// Push a single dependency.
    ///
    pub fn push(&mut self, dependency: String, is_runtime_code: bool) {
        if dependency == self.identifier || self.inner.contains(&dependency) {
            return;
        }

        if is_runtime_code {
            self.inner.insert(0, dependency);
        } else {
            self.inner.push(dependency);
        }
    }
}
