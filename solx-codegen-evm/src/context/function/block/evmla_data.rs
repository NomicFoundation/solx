//!
//! The LLVM function block EVM legacy assembly data.
//!

///
/// The LLVM function block EVM legacy assembly data.
///
/// Describes some data that is only relevant to the EVM legacy assembly.
///
#[derive(Debug, Clone)]
pub struct EVMLAData {
    /// The hash of the block's initial stack state.
    pub stack_hash: u64,
}

impl EVMLAData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(stack_hash: u64) -> Self {
        Self { stack_hash }
    }
}
