//!
//! The block-local shadow stack slot.
//!

///
/// The block-local shadow stack slot.
///
/// While a block is translated, each stack position either still lives in the function-frame alloca
/// it inherited from the predecessor or holds an SSA value produced within the block. The value is
/// written back to memory only at block exits.
///
#[derive(Debug, Clone, Copy)]
pub enum ShadowSlot<'ctx> {
    /// The value equals the function-frame alloca at the given index.
    Memory(usize),
    /// The value is the given SSA operand, not yet written back to the frame.
    Value(inkwell::values::BasicValueEnum<'ctx>),
}
