//!
//! Resolving one segment's nodes to MLIR locations.
//!

use std::collections::HashMap;
use std::ops::Range;

use melior::ir::Location;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::compilation::FileId;

use solx_mlir::DebugInfoFileName;
use solx_utils::LineIndex;

use crate::debug_locations::sources::Sources;

/// Resolves one segment's nodes to MLIR locations through the compilation's sources.
pub struct Resolver<'context> {
    /// The MLIR context the locations are built in.
    melior: &'context melior::Context,
    /// The compilation's sources.
    sources: &'context Sources<'context>,
    /// Each source a node has been resolved in, named on first use.
    named_sources: HashMap<FileId, NamedSource<'context>>,
}

/// A source's line index, and the name its locations reference.
struct NamedSource<'context> {
    /// The source's line index.
    line_index: &'context LineIndex,
    /// The name its locations reference.
    name: DebugInfoFileName<'context>,
}

impl<'context> Resolver<'context> {
    /// Resolves through `sources` in `melior`.
    pub fn new(melior: &'context melior::Context, sources: &'context Sources<'context>) -> Self {
        Self {
            melior,
            sources,
            named_sources: HashMap::new(),
        }
    }

    /// The MLIR location of `node`'s first byte.
    pub fn location(&mut self, node: &impl NodeLocation) -> Location<'context> {
        let (file_id, bytes) = Self::source_range(node);
        self.location_in(file_id, bytes.start)
    }

    /// The MLIR location of `node`'s last byte.
    pub fn location_end(&mut self, node: &impl NodeLocation) -> Location<'context> {
        let (file_id, bytes) = Self::source_range(node);
        self.location_in(file_id, bytes.end - 1)
    }

    /// The source and byte range of `node`. Slang gives both to every node other than an empty
    /// collection or a variant without source text, and the lowering takes no position from those.
    pub fn source_range(node: &impl NodeLocation) -> (&FileId, Range<usize>) {
        (
            node.calculate_file_id()
                .expect("slang locates every node the lowering takes a position from"),
            node.calculate_text_range()
                .expect("slang locates every node the lowering takes a position from"),
        )
    }

    /// The MLIR location of byte `offset` in the source `file_id` names, which is named the first
    /// time a node is resolved in it.
    fn location_in(&mut self, file_id: &FileId, offset: usize) -> Location<'context> {
        if let Some(named) = self.named_sources.get(file_id) {
            return named.location(offset);
        }
        let named = NamedSource {
            line_index: self.sources.line_index(file_id),
            name: DebugInfoFileName::new(self.melior, file_id.as_str()),
        };
        let location = named.location(offset);
        self.named_sources.insert(file_id.clone(), named);
        location
    }
}

impl<'context> NamedSource<'context> {
    /// The MLIR location of byte `offset`.
    fn location(&self, offset: usize) -> Location<'context> {
        let (line, column) = self.line_index.line_and_column(offset);
        self.name.location(line, column)
    }
}
