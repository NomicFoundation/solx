//!
//! The compilation's source texts.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::OnceLock;

use slang_solidity_v2::compilation::FileId;

use solx_utils::LineIndex;

/// Every source in the compilation, keyed by file. A source is indexed the first time any object's
/// resolver needs it, and the index is shared by every object after it.
pub struct Sources<'source> {
    /// Each source.
    sources: HashMap<&'source FileId, Source<'source>>,
}

/// A source's text and its line index.
struct Source<'source> {
    /// The source's text.
    text: &'source str,
    /// The source's line index, built on first use.
    line_index: OnceLock<LineIndex>,
}

impl<'source> Sources<'source> {
    /// Wraps the compilation's source `texts`.
    pub fn new(texts: &'source BTreeMap<FileId, &'source str>) -> Self {
        Self {
            sources: texts
                .iter()
                .map(|(file_id, text)| {
                    let source = Source {
                        text,
                        line_index: OnceLock::new(),
                    };
                    (file_id, source)
                })
                .collect(),
        }
    }

    /// The line index of the source `file_id` names.
    pub fn line_index(&self, file_id: &FileId) -> &LineIndex {
        let source = self
            .sources
            .get(file_id)
            .expect("every node's file is a source of the compilation");
        source
            .line_index
            .get_or_init(|| LineIndex::new(source.text))
    }
}
