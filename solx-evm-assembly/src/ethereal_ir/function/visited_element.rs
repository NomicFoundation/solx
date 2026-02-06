//!
//! The Ethereal IR block visited element.
//!

use std::cmp::Ordering;

///
/// The Ethereal IR block visited element.
///
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VisitedElement {
    /// The block key.
    pub block_key: solx_codegen_evm::BlockKey,
    /// The initial stack state hash.
    pub stack_hash: u64,
}

impl VisitedElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(block_key: solx_codegen_evm::BlockKey, stack_hash: u64) -> Self {
        Self {
            block_key,
            stack_hash,
        }
    }
}

impl PartialOrd for VisitedElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VisitedElement {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.block_key.code_segment, other.block_key.code_segment) {
            (solx_utils::CodeSegment::Deploy, solx_utils::CodeSegment::Runtime) => Ordering::Less,
            (solx_utils::CodeSegment::Runtime, solx_utils::CodeSegment::Deploy) => {
                Ordering::Greater
            }
            (solx_utils::CodeSegment::Deploy, solx_utils::CodeSegment::Deploy)
            | (solx_utils::CodeSegment::Runtime, solx_utils::CodeSegment::Runtime) => {
                let tag_comparison = self.block_key.tag.cmp(&other.block_key.tag);
                if tag_comparison == Ordering::Equal {
                    self.stack_hash.cmp(&other.stack_hash)
                } else {
                    tag_comparison
                }
            }
        }
    }
}
