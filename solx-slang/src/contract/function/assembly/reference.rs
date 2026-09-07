//!
//! What a Yul path denotes: a pointer to read and write, or a read-only word, and the field a
//! suffixed path projects out of a Solidity declaration.
//!

use slang_solidity_v2::ast::BuiltIn;

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

/// The field a suffixed Yul path projects out of a Solidity declaration.
#[derive(Clone, Copy)]
pub enum YulField {
    /// `.slot` of a storage reference or state variable.
    Slot,
    /// `.offset` of a storage or calldata reference.
    Offset,
    /// `.length` of a calldata reference.
    Length,
    /// `.selector` of an external function pointer.
    Selector,
    /// `.address` of an external function pointer.
    Address,
}

impl From<BuiltIn> for YulField {
    fn from(built_in: BuiltIn) -> Self {
        match built_in {
            BuiltIn::YulSlot => Self::Slot,
            BuiltIn::YulOffset => Self::Offset,
            BuiltIn::YulLengthField => Self::Length,
            BuiltIn::YulSelector => Self::Selector,
            BuiltIn::YulAddressField => Self::Address,
            built_in => unreachable!("{built_in:?} is not a Yul path suffix"),
        }
    }
}
