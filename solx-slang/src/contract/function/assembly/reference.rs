//!
//! What a Yul path denotes: a pointer to read and write, or a read-only word.
//!

use solx_mlir::Context;
use solx_mlir::Pointer;
use solx_mlir::Word;

/// The two shapes a Yul name resolves to. A Yul variable and a Solidity variable bridged in are
/// pointers; the compile-time `.slot` / `.offset` of a state variable is a bare word, which is why
/// Solidity rejects assigning to one.
#[derive(Clone, Copy)]
pub enum YulReference<'context> {
    /// A pointer, read through `yul.load` and written through `yul.store`.
    Pointer(Pointer<'context>),
    /// A word fixed at compile time, read-only.
    Word(Word<'context>),
}

impl<'context> YulReference<'context> {
    /// The word this reference reads as.
    pub fn read(self, context: &Context<'context>) -> Word<'context> {
        match self {
            Self::Pointer(pointer) => pointer.load(context),
            Self::Word(word) => word,
        }
    }

    /// Writes `value` through this reference.
    pub fn write(self, value: Word<'context>, context: &Context<'context>) {
        match self {
            Self::Pointer(pointer) => pointer.store(value, context),
            Self::Word(_) => {
                unreachable!("slang rejects assigning to a compile-time Yul reference")
            }
        }
    }
}
