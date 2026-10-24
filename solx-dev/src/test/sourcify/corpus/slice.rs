//!
//! `solx` Sourcify sweep corpus slice.
//!

use std::num::NonZeroUsize;

///
/// A round-robin slice of the sorted corpus, so CI shards split it evenly.
///
#[derive(Debug)]
pub struct Slice {
    /// The number of shards.
    pub shard_count: NonZeroUsize,
    /// The shard to keep.
    pub shard_index: usize,
    /// The maximum number of contracts to keep, if any.
    pub limit: Option<usize>,
}

impl Slice {
    ///
    /// Sorts the corpus file names and keeps this slice of them.
    ///
    pub fn select(&self, mut names: Vec<String>) -> Vec<String> {
        names.sort();
        names
            .into_iter()
            .enumerate()
            .filter(|(index, _name)| index % self.shard_count.get() == self.shard_index)
            .map(|(_index, name)| name)
            .take(self.limit.unwrap_or(usize::MAX))
            .collect()
    }
}
