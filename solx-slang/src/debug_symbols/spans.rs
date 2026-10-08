//!
//! The ranges of a source unit's nonterminal nodes.
//!

use std::collections::BTreeSet;

use slang_solidity_v2::ast;
use slang_solidity_v2::ast::NodeLocation;
use slang_solidity_v2::ast::SourceUnit;
use slang_solidity_v2::ast::visitor::Visitor;

/// The `[offset, length]` of every node visited that covers text, ordered and deduplicated.
#[derive(Default)]
pub struct Spans(BTreeSet<[usize; 2]>);

impl Spans {
    /// Records a node's range, if it covers any text. A modifier without attributes has an empty
    /// attribute node at offset 0.
    fn record(&mut self, node: &dyn NodeLocation) -> bool {
        if let Some(range) = node
            .calculate_text_range()
            .filter(|range| !range.is_empty())
        {
            self.0.insert([range.start, range.len()]);
        }
        true
    }

    /// Collects the spans of every sequence and non-empty collection node in `unit`. Choices are
    /// skipped: each spans its variant, which is either recorded itself or a terminal.
    pub fn collect(unit: &SourceUnit) -> Vec<[usize; 2]> {
        let mut spans = Self::default();
        ast::visitor::accept_source_unit(unit, &mut spans);
        spans.0.into_iter().collect()
    }
}

impl Visitor for Spans {
    ast::visitor::impl_nonterminal_visitor_hooks!(enter = record);
}
