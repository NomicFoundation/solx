//!
//! What a Yul path resolves to: a slot to read and write, or a word that is read-only.
//!

use crate::Context;
use crate::Slot;
use crate::Word;

/// The two shapes a Yul name resolves to. A Yul variable and a Solidity variable bridged in are
/// slots; the compile-time `.slot` / `.offset` of a state variable is a bare word, which is why
/// Solidity rejects assigning to one.
#[derive(Clone, Copy)]
pub enum YulReference<'context> {
    /// A slot, read through `yul.load` and written through `yul.store`.
    Slot(Slot<'context>),
    /// A word fixed at compile time, read-only.
    Word(Word<'context>),
}

impl<'context> YulReference<'context> {
    /// The word this reference reads as.
    pub fn read(self, context: &Context<'context>) -> Word<'context> {
        match self {
            Self::Slot(slot) => slot.load(context),
            Self::Word(word) => word,
        }
    }

    /// Writes `value` through this reference.
    pub fn write(self, value: Word<'context>, context: &Context<'context>) {
        match self {
            Self::Slot(slot) => slot.store(value, context),
            Self::Word(_) => {
                unreachable!("slang rejects assigning to a compile-time Yul reference")
            }
        }
    }
}

impl<'context> From<Slot<'context>> for YulReference<'context> {
    fn from(slot: Slot<'context>) -> Self {
        Self::Slot(slot)
    }
}

impl<'context> From<Word<'context>> for YulReference<'context> {
    fn from(word: Word<'context>) -> Self {
        Self::Word(word)
    }
}
