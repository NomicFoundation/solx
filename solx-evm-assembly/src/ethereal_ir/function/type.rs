//!
//! The Ethereal IR function type.
//!

///
/// The Ethereal IR function type.
///
#[derive(Debug)]
pub enum Type {
    /// Entry function at the beginning of the contract.
    Entry,
    /// Defined function called from entry function or another defined function.
    Defined {
        /// The function name.
        name: String,
        /// The function AST node ID.
        ast_id: Option<usize>,
        /// The function initial block key.
        block_key: solx_codegen_evm::BlockKey,
        /// The size of stack input (in cells or 256-bit words).
        input_size: usize,
        /// The size of stack output (in cells or 256-bit words).
        output_size: usize,
    },
}

impl Type {
    ///
    /// A shortcut constructor.
    ///
    pub fn new_entry() -> Self {
        Self::Entry
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_defined(
        name: String,
        ast_id: Option<usize>,
        block_key: solx_codegen_evm::BlockKey,
        input_size: usize,
        output_size: usize,
    ) -> Self {
        Self::Defined {
            name,
            ast_id,
            block_key,
            input_size,
            output_size,
        }
    }
}
