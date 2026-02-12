//!
//! The Ethereal IR block visited element.
//!

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
