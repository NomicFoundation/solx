//!
//! The LLVM IR EVMLA data trait.
//!

pub mod shadow_slot;

use crate::context::value::Value;

use self::shadow_slot::ShadowSlot;

///
/// The LLVM IR EVMLA data trait.
///
pub trait IEVMLAData<'ctx> {
    ///
    /// Returns the element from the specified stack position.
    ///
    /// # Panics
    /// If `position` is out of bounds.
    ///
    fn get_element(&self, position: usize) -> &Value<'ctx>;

    ///
    /// Resets the shadow stack to the identity mapping for a freshly entered block.
    ///
    fn shadow_reset(&mut self);

    ///
    /// Returns the shadow slot at the specified stack position.
    ///
    /// # Panics
    /// If `position` is out of bounds.
    ///
    fn shadow_peek(&self, position: usize) -> ShadowSlot<'ctx>;

    ///
    /// Binds an SSA value to the specified stack position.
    ///
    /// # Panics
    /// If `position` is out of bounds.
    ///
    fn shadow_write(&mut self, position: usize, value: inkwell::values::BasicValueEnum<'ctx>);

    ///
    /// Duplicates the source shadow slot onto the destination position.
    ///
    /// # Panics
    /// If either position is out of bounds.
    ///
    fn shadow_dup(&mut self, destination: usize, source: usize);

    ///
    /// Swaps two shadow slots.
    ///
    /// # Panics
    /// If either position is out of bounds.
    ///
    fn shadow_swap(&mut self, first: usize, second: usize);
}
